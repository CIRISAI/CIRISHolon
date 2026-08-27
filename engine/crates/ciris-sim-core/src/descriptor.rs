//! Descriptor-as-generator materialization — accurate decomposition drawn from the
//! descriptor holon's own declared statistics (INTEGRATION_FRAME.md).
//!
//! The demo materializer halved gross counts and produced children with no mineral
//! identity, no sizes, no flaws, because the parent carried no information about what
//! its interior IS. This module closes that gap without any new entity class:
//!
//! * **The descriptor holon is the generator.** A stone descriptor is an ordinary
//!   [`RuntimeArena`] holon. Its mineral children's REG+ gross ledgers carry the modal
//!   fractions as exact integers ([`expand_stone_descriptor`]), so the existing
//!   transactional conservation check enforces modal composition for free.
//! * **Ensemble distributions are whole-only state.** Grain-size log-normal
//!   `(mu, sigma)` and weakest-link Weibull `(m, sigma_0, flaw density)` are facts about
//!   the ensemble that do not factor through any single materialized child; they live in
//!   the stone descriptor's `whole` slots ([`DrawParams`]), using the variable-width
//!   whole pool exactly as intended.
//! * **"Made of" stays [`MaterialBinding`].** The materializer reads the binding,
//!   walks the descriptor subtree, and draws subject children deterministically.
//!
//! ## Quenched realization (M28/A1)
//!
//! The draw is seeded from a seed **persisted on the subject holon** in two whole-state
//! slots ([`encode_seed`]; each slot holds 32 bits as an exactly-representable integer).
//! Those slots are **Record-carrying**: they are written when the holon is created —
//! the root subject by the scene builder, every materialized child by this module — and
//! are never resampled, so replay can never re-roll the wall's strength. Decomposing
//! the same wall twice yields bit-identical children. Every draw is
//! `EntropyProvenance::Seeded` in the sense of [`crate::entropy`]: the PRNG is a
//! hand-rolled SplitMix64 ([`SplitMix64`]) implementing [`EntropySource`], never an OS
//! or hardware source.
//!
//! ## The statistical-composition certificate
//!
//! Ledger exactness was already enforced. Accurate decomposition adds one obligation:
//! the empirical distribution of a materialized ensemble must converge to the
//! descriptor's declared distributions. [`certify_grains`] checks exact pivotal
//! statistics — under the declared law, `z_i = (ln d_i - mu)/sigma` is exactly N(0,1)
//! and `t_i = lambda * V_i * (sigma_i/sigma_0)^m` is exactly Exp(1) — so every check
//! compares an empirical mean to a known expectation and variance.
//!
//! **Tolerance scaling rule (documented, per the integration frame):** each check
//! passes iff `|empirical mean - expectation| <= Z * sqrt(Var / n)` with `Z = 5`
//! ([`CERT_Z`]); the composition checks add `1/n` for integer rounding. First-moment
//! checks (per-mineral fraction, mean `z`, mean `ln t`, mean `t`) resolve from
//! `n >= 8` ([`CERT_FLOOR_MEAN`]); second-moment checks (`z^2`, `t^2`) resolve from
//! `n >= 32` ([`CERT_FLOOR_SPREAD`]) because their small-`n` tails are visibly
//! non-normal. A check below its floor is recorded **unresolved** — neither support
//! nor violation — so 288 residents are tested exactly as hard as 288 draws can
//! discriminate and no harder; a Weibull tail only 10^6 grains would show is left to
//! ensembles of that size. A materialization whose batch fails a resolved check is
//! REJECTED before anything is committed, exactly like a ledger violation.
//!
//! The [`RuntimeMaterializer`] trait fixes the error type to [`HolonError`], which is
//! exhaustively matched by the out-of-workspace component adapter; a statistical
//! rejection therefore surfaces through the trait as
//! [`HolonError::GrossStateDoesNotCompose`] (the certificate IS the downward
//! composition check), while the distinguishing [`StatisticalReport`] is retained on
//! the materializer ([`DescriptorMaterializer::last_report`]) and on the richer
//! [`DescriptorMaterializer::materialize_described`] error path.

use alloc::vec;
use alloc::vec::Vec;
use core::f64::consts::PI;
use core::ops::Range;

use crate::entropy::EntropySource;
use crate::holon::{Channels, Decomposition, HolonError};
use crate::material::MaterialBinding;
use crate::regplus::GrossState;
use crate::runtime::{
    RuntimeArena, RuntimeArenaBuilder, RuntimeHolon, RuntimeHolonSpec, RuntimeMaterializer,
    NO_RUNTIME_HOLON,
};

/// Hand-rolled SplitMix64. Deterministic on every IEEE-754/two's-complement target,
/// no dependencies, and an [`EntropySource`] so the engine's seeded-provenance idiom
/// applies unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }
}

impl EntropySource for SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// Uniform on the OPEN interval (0, 1): safe to pass to `ln`. The half-offset keeps
/// every implementor bit-identical, mirroring [`EntropySource::next_f64`].
fn next_open01(source: &mut impl EntropySource) -> f64 {
    ((source.next_u64() >> 11) as f64 + 0.5) * (1.0 / 9_007_199_254_740_992.0)
}

/// Number of whole-state slots the quenched-realization seed occupies on a subject.
pub const SEED_SLOTS: usize = 2;

/// Encode a 64-bit seed into two whole-state scalars, 32 bits each. Every value is an
/// integer below 2^32 and therefore exactly representable; no NaN payloads are ever
/// stored in the whole pool.
pub const fn encode_seed(seed: u64) -> [f64; SEED_SLOTS] {
    [(seed >> 32) as f64, (seed & 0xFFFF_FFFF) as f64]
}

fn exact_u32(value: f64) -> Option<u32> {
    if (0.0..4_294_967_296.0).contains(&value) && (value as u32) as f64 == value {
        Some(value as u32)
    } else {
        None
    }
}

/// Decode a seed persisted by [`encode_seed`]. `None` means the slots do not hold a
/// Record-carrying seed (wrong width, negative, fractional, or out of range).
pub fn decode_seed(slots: &[f64]) -> Option<u64> {
    if slots.len() < SEED_SLOTS {
        return None;
    }
    let hi = exact_u32(slots[0])?;
    let lo = exact_u32(slots[1])?;
    Some(((hi as u64) << 32) | lo as u64)
}

/// Whole-state layout of a stone descriptor holon: the ensemble distributions that do
/// not factor through any single child. Slot indices are the public contract.
pub const STONE_WHOLE_LEN: usize = 5;
pub const STONE_GRAIN_MU: usize = 0;
pub const STONE_GRAIN_SIGMA: usize = 1;
pub const STONE_WEIBULL_M: usize = 2;
pub const STONE_WEIBULL_SIGMA0: usize = 3;
pub const STONE_FLAW_DENSITY: usize = 4;

/// Whole-state layout of a materialized grain child. Slots 0..2 are the child's own
/// Record-carrying quenched seed (so the child can itself be materialized later
/// without resampling anything already realized); the rest are its realized values.
pub const GRAIN_WHOLE_LEN: usize = 5;
pub const GRAIN_SEED_HI: usize = 0;
pub const GRAIN_SEED_LO: usize = 1;
pub const GRAIN_MINERAL: usize = 2;
pub const GRAIN_DIAMETER_M: usize = 3;
pub const GRAIN_STRENGTH_PA: usize = 4;

/// Declared ensemble law carried in a stone descriptor's whole-only state.
///
/// Grain diameters are log-normal: `ln d ~ N(grain_mu_ln_m, grain_sigma_ln^2)`.
/// Quenched strength is the weakest-link draw over the grain's own volume
/// `V = (pi/6) d^3` at flaw density `lambda`:
/// `P(strength > s) = exp(-lambda * V * (s / sigma_0)^m)`,
/// which uses all three Weibull parameters in one principled draw and carries the
/// size effect automatically.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawParams {
    pub grain_mu_ln_m: f64,
    pub grain_sigma_ln: f64,
    pub weibull_m: f64,
    pub weibull_sigma0_pa: f64,
    pub flaw_density_per_m3: f64,
}

impl DrawParams {
    pub fn validate(&self) -> bool {
        self.grain_mu_ln_m.is_finite()
            && self.grain_sigma_ln.is_finite()
            && self.grain_sigma_ln > 0.0
            && self.weibull_m.is_finite()
            && self.weibull_m > 0.0
            && self.weibull_sigma0_pa.is_finite()
            && self.weibull_sigma0_pa > 0.0
            && self.flaw_density_per_m3.is_finite()
            && self.flaw_density_per_m3 > 0.0
    }

    pub const fn to_whole(&self) -> [f64; STONE_WHOLE_LEN] {
        [
            self.grain_mu_ln_m,
            self.grain_sigma_ln,
            self.weibull_m,
            self.weibull_sigma0_pa,
            self.flaw_density_per_m3,
        ]
    }

    pub fn from_whole(whole: &[f64]) -> Option<Self> {
        if whole.len() < STONE_WHOLE_LEN {
            return None;
        }
        let params = Self {
            grain_mu_ln_m: whole[STONE_GRAIN_MU],
            grain_sigma_ln: whole[STONE_GRAIN_SIGMA],
            weibull_m: whole[STONE_WEIBULL_M],
            weibull_sigma0_pa: whole[STONE_WEIBULL_SIGMA0],
            flaw_density_per_m3: whole[STONE_FLAW_DENSITY],
        };
        if params.validate() {
            Some(params)
        } else {
            None
        }
    }
}

/// One realized grain: the quenched draw a materialized subject child carries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrainDraw {
    /// The child's own Record-carrying seed (persisted; drives any later refinement).
    pub seed: u64,
    /// Index of the mineral descriptor child within the descriptor subtree.
    pub mineral: usize,
    pub diameter_m: f64,
    pub strength_pa: f64,
}

impl GrainDraw {
    pub fn to_whole(&self) -> [f64; GRAIN_WHOLE_LEN] {
        let seed = encode_seed(self.seed);
        [
            seed[0],
            seed[1],
            self.mineral as f64,
            self.diameter_m,
            self.strength_pa,
        ]
    }

    pub fn from_whole(whole: &[f64]) -> Option<Self> {
        if whole.len() < GRAIN_WHOLE_LEN {
            return None;
        }
        let seed = decode_seed(&whole[GRAIN_SEED_HI..=GRAIN_SEED_LO])?;
        let mineral = whole[GRAIN_MINERAL];
        if mineral < 0.0 || (mineral as usize) as f64 != mineral {
            return None;
        }
        Some(Self {
            seed,
            mineral: mineral as usize,
            diameter_m: whole[GRAIN_DIAMETER_M],
            strength_pa: whole[GRAIN_STRENGTH_PA],
        })
    }
}

/// Read a materialized grain child's realized values back from the arena.
pub fn read_grain(arena: &RuntimeArena, holon: usize) -> Option<GrainDraw> {
    GrainDraw::from_whole(arena.whole_state(holon)?)
}

// ---------------------------------------------------------------------------
// Exact integer splits. The ledger is never approximated: every split composes
// back to the parent exactly, so `RuntimeArena::materialize`'s conservation
// check can do its job instead of being routed around.
// ---------------------------------------------------------------------------

const fn share_u64(total: u64, parts: u64, index: u64) -> u64 {
    let base = total / parts;
    if index < total % parts {
        base + 1
    } else {
        base
    }
}

const fn share_i64(total: i64, parts: i64, index: i64) -> i64 {
    let base = total.div_euclid(parts);
    if index < total.rem_euclid(parts) {
        base + 1
    } else {
        base
    }
}

/// Largest-remainder apportionment of `total` over integer `weights`: exact
/// (`sum == total`), deterministic (ties broken by lowest index). Weights must each
/// be positive and fit in `u32` so the `i128` intermediate cannot overflow.
fn weighted_shares(total: i128, weights: &[u64]) -> Vec<i128> {
    let weight_total: i128 = weights.iter().map(|w| *w as i128).sum();
    let mut shares: Vec<i128> = Vec::with_capacity(weights.len());
    let mut remainders: Vec<i128> = Vec::with_capacity(weights.len());
    let mut assigned: i128 = 0;
    for weight in weights {
        let product = total * *weight as i128;
        let base = product.div_euclid(weight_total);
        remainders.push(product.rem_euclid(weight_total));
        assigned += base;
        shares.push(base);
    }
    let mut deficit = total - assigned;
    while deficit > 0 {
        let mut best = 0;
        for (k, remainder) in remainders.iter().enumerate() {
            if *remainder > remainders[best] {
                best = k;
            }
        }
        shares[best] += 1;
        remainders[best] = -1;
        deficit -= 1;
    }
    shares
}

fn weighted_gross(total: GrossState, weights: &[u64]) -> Vec<GrossState> {
    let constituents = weighted_shares(total.constituents as i128, weights);
    let occupancy = weighted_shares(total.occupancy as i128, weights);
    let momentum_x = weighted_shares(total.momentum[0] as i128, weights);
    let momentum_y = weighted_shares(total.momentum[1] as i128, weights);
    (0..weights.len())
        .map(|k| {
            GrossState::aggregate(
                constituents[k] as u64,
                occupancy[k] as u64,
                [momentum_x[k] as i64, momentum_y[k] as i64],
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// The descriptor subtree: the stone descriptor decomposes like everything else.
// ---------------------------------------------------------------------------

/// Materialize a stone descriptor's mineral children. `weights` are the modal weights
/// (for example `[30, 60, 10]` for quartz/feldspar/mica); each mineral's gross ledger
/// receives its largest-remainder integer share of the parent's constituent count, so
/// the shares compose exactly and the existing transactional conservation check
/// enforces modal composition for free. Mineral identity is the child's ordinal.
pub fn expand_stone_descriptor(
    arena: &mut RuntimeArena,
    descriptor: usize,
    weights: &[u64],
) -> Result<Range<u32>, HolonError> {
    let record = *arena.holon(descriptor).ok_or(HolonError::InvalidParent)?;
    if weights.is_empty() || weights.iter().any(|w| *w == 0 || *w > u32::MAX as u64) {
        return Err(HolonError::InvalidDecomposition);
    }
    let gross = weighted_gross(record.gross, weights);
    let depth = record
        .depth
        .checked_add(1)
        .ok_or(HolonError::InvalidDepth)?;
    let specs: Vec<RuntimeHolonSpec<'_>> = gross
        .iter()
        .map(|share| RuntimeHolonSpec {
            parent: descriptor as u32,
            depth,
            grain_units: 1,
            gross: *share,
            whole: &[],
            channels: record.channels,
            boundary: false,
            decomposition: Decomposition::Leaf,
        })
        .collect();
    arena.materialize(descriptor, &specs)
}

/// Build a self-contained descriptor holarchy: a stone descriptor root carrying `law`
/// in its whole-only state, expanded into mineral Leaf children carrying the modal
/// `weights` in their gross ledgers. The descriptor lives in its own arena — a
/// descriptor library is its own holarchy root, not a constituent of the scene it
/// describes (placing it under a scene root would double-count the scene's ledger).
pub fn build_stone_descriptor(
    total_constituents: u64,
    weights: &[u64],
    law: &DrawParams,
) -> Result<RuntimeArena, HolonError> {
    let whole = law.to_whole();
    let mut builder = RuntimeArenaBuilder::with_capacity(1 + weights.len(), STONE_WHOLE_LEN);
    builder.push(RuntimeHolonSpec {
        parent: NO_RUNTIME_HOLON,
        depth: 0,
        grain_units: 2,
        gross: GrossState::aggregate(total_constituents, 0, [0, 0]),
        whole: &whole,
        channels: Channels::REG_PLUS,
        boundary: false,
        decomposition: Decomposition::Latent,
    })?;
    let root = 0;
    let mut arena = builder.build(root)?;
    expand_stone_descriptor(&mut arena, root as usize, weights)?;
    Ok(arena)
}

/// The declared law read back from a descriptor subtree: whole-only distribution
/// parameters plus the modal fractions the mineral children's ledgers carry.
#[derive(Clone, Debug, PartialEq)]
pub struct StoneDescriptorView {
    pub params: DrawParams,
    /// Exact integer constituent counts, one per mineral child, in child-id order.
    pub counts: Vec<u64>,
    /// `counts` normalized by the descriptor's constituent count.
    pub fractions: Vec<f64>,
}

/// Errors of the descriptor-as-generator path. The [`RuntimeMaterializer`] trait can
/// only surface [`HolonError`]; this richer type is available through
/// [`DescriptorMaterializer::materialize_described`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DescriptorError {
    Holon(HolonError),
    /// The descriptor's mineral children are not resident, so there is nothing to
    /// generate from.
    DescriptorNotExpanded,
    /// The descriptor's whole-only slots do not hold a valid [`DrawParams`].
    DescriptorLawInvalid,
    /// The subject holon carries no Record-carrying quenched seed (M28/A1): refusing
    /// to invent one is what makes replay unable to resample the wall's strength.
    SeedMissing,
    /// Fanout below 2 or other unusable configuration.
    InvalidConfiguration,
    /// The drawn batch failed the statistical-composition certificate; nothing was
    /// committed.
    Statistics(StatisticalReport),
}

impl From<HolonError> for DescriptorError {
    fn from(error: HolonError) -> Self {
        Self::Holon(error)
    }
}

/// Read the generator content of a descriptor subtree: the whole-only law off the
/// root, the modal fractions off the mineral children's gross ledgers.
pub fn read_stone_descriptor(
    arena: &RuntimeArena,
    descriptor: usize,
) -> Result<StoneDescriptorView, DescriptorError> {
    let record = *arena
        .holon(descriptor)
        .ok_or(DescriptorError::Holon(HolonError::InvalidParent))?;
    if record.decomposition != Decomposition::Expanded {
        return Err(DescriptorError::DescriptorNotExpanded);
    }
    let params = arena
        .whole_state(descriptor)
        .and_then(DrawParams::from_whole)
        .ok_or(DescriptorError::DescriptorLawInvalid)?;
    let total = record.gross.constituents;
    if total == 0 {
        return Err(DescriptorError::DescriptorLawInvalid);
    }
    let mut counts = Vec::new();
    for (id, holon) in arena.holons().iter().enumerate() {
        if holon.parent == descriptor as u32 && id != descriptor {
            counts.push(holon.gross.constituents);
        }
    }
    let fractions = counts
        .iter()
        .map(|count| *count as f64 / total as f64)
        .collect();
    Ok(StoneDescriptorView {
        params,
        counts,
        fractions,
    })
}

// ---------------------------------------------------------------------------
// The deterministic draw.
// ---------------------------------------------------------------------------

/// Draw one grain from its own persisted seed. Consumes exactly four PRNG outputs
/// (mineral, two for Box-Muller, one for the weakest-link exceedance) so the draw is
/// a pure function of `(child_seed, law, fractions)`.
pub fn draw_grain(child_seed: u64, law: &DrawParams, fractions: &[f64]) -> GrainDraw {
    let mut stream = SplitMix64::new(child_seed);
    let u_mineral = next_open01(&mut stream);
    let mut mineral = fractions.len().saturating_sub(1);
    let mut cumulative = 0.0;
    for (k, fraction) in fractions.iter().enumerate() {
        cumulative += fraction;
        if u_mineral < cumulative {
            mineral = k;
            break;
        }
    }
    let u1 = next_open01(&mut stream);
    let u2 = next_open01(&mut stream);
    let z = libm::sqrt(-2.0 * libm::log(u1)) * libm::cos(2.0 * PI * u2);
    let diameter_m = libm::exp(law.grain_mu_ln_m + law.grain_sigma_ln * z);
    let exceedance = -libm::log(next_open01(&mut stream));
    let volume_m3 = PI / 6.0 * diameter_m * diameter_m * diameter_m;
    let strength_pa = law.weibull_sigma0_pa
        * libm::pow(
            exceedance / (law.flaw_density_per_m3 * volume_m3),
            1.0 / law.weibull_m,
        );
    GrainDraw {
        seed: child_seed,
        mineral,
        diameter_m,
        strength_pa,
    }
}

/// Draw a batch of `n` grains from a parent's persisted seed. Child seed `i` is the
/// `(i+1)`-th SplitMix64 output of the parent stream, so children are independent
/// streams and the whole batch is a pure function of the parent seed.
pub fn draw_grains(parent_seed: u64, law: &DrawParams, fractions: &[f64], n: usize) -> Vec<GrainDraw> {
    let mut parent_stream = SplitMix64::new(parent_seed);
    (0..n)
        .map(|_| draw_grain(parent_stream.next_u64(), law, fractions))
        .collect()
}

// ---------------------------------------------------------------------------
// The statistical-composition certificate.
// ---------------------------------------------------------------------------

/// Tolerance multiplier: a resolved check fires only past `Z` standard errors.
pub const CERT_Z: f64 = 5.0;
/// Below this count, first-moment checks are declared unresolvable.
pub const CERT_FLOOR_MEAN: usize = 8;
/// Below this count, second-moment checks are declared unresolvable (their small-`n`
/// tails are chi-square-shaped, not normal; the `Z` rule would over-fire).
pub const CERT_FLOOR_SPREAD: usize = 32;

const EULER_GAMMA: f64 = 0.577_215_664_901_532_9;

/// Which certificate check failed (or would be reported on).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CertCheck {
    /// A draw carried a non-finite/non-positive value or an out-of-range mineral.
    /// Fails at any `n`.
    InvalidDraw,
    /// A drawn mineral has declared fraction zero. Fails at any `n`.
    MineralSupport,
    /// Empirical modal fraction of mineral `k` outside its binomial band.
    MineralFraction(usize),
    /// Mean of `z = (ln d - mu)/sigma`, expectation 0, variance 1.
    SizeMean,
    /// Mean of `z^2`, expectation 1, variance 2.
    SizeSpread,
    /// Mean of `ln t`, expectation `-gamma`, variance `pi^2/6` — the classic
    /// Weibull-modulus discriminator.
    StrengthLogMean,
    /// Mean of `t = lambda V (sigma/sigma_0)^m`, expectation 1, variance 1.
    StrengthMean,
    /// Mean of `t^2`, expectation 2, variance 20.
    StrengthSpread,
}

/// Outcome of [`certify_grains`]. `failed == None` means no resolved check fired;
/// `unresolved` says how many checks the sample size could not discriminate — a
/// small-`n` pass is a statement about resolution, not a certificate of the tail.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatisticalReport {
    pub n: usize,
    pub passed: u32,
    pub unresolved: u32,
    pub failed: Option<CertCheck>,
}

impl StatisticalReport {
    pub const fn rejected(&self) -> bool {
        self.failed.is_some()
    }
}

fn apply_check(
    report: &mut StatisticalReport,
    floor: usize,
    deviation: f64,
    tolerance: f64,
    tag: CertCheck,
) {
    if report.failed.is_some() {
        return;
    }
    if report.n < floor {
        report.unresolved += 1;
    } else if deviation <= tolerance {
        report.passed += 1;
    } else {
        report.failed = Some(tag);
    }
}

/// Certify a materialized ensemble against the DECLARED law (always the descriptor's,
/// never the generator's own configuration — that is what gives the gate teeth).
/// See the module docs for the tolerance scaling rule.
pub fn certify_grains(
    draws: &[GrainDraw],
    declared: &DrawParams,
    fractions: &[f64],
) -> StatisticalReport {
    let n = draws.len();
    let mut report = StatisticalReport {
        n,
        passed: 0,
        unresolved: 0,
        failed: None,
    };
    let mut counts = vec![0_usize; fractions.len()];
    let mut sum_z = 0.0;
    let mut sum_z2 = 0.0;
    let mut sum_t = 0.0;
    let mut sum_t2 = 0.0;
    let mut sum_ln_t = 0.0;
    for draw in draws {
        let valid = draw.diameter_m.is_finite()
            && draw.diameter_m > 0.0
            && draw.strength_pa.is_finite()
            && draw.strength_pa > 0.0
            && draw.mineral < fractions.len();
        if !valid {
            report.failed = Some(CertCheck::InvalidDraw);
            return report;
        }
        if fractions[draw.mineral] <= 0.0 {
            report.failed = Some(CertCheck::MineralSupport);
            return report;
        }
        counts[draw.mineral] += 1;
        let z = (libm::log(draw.diameter_m) - declared.grain_mu_ln_m) / declared.grain_sigma_ln;
        sum_z += z;
        sum_z2 += z * z;
        let volume = PI / 6.0 * draw.diameter_m * draw.diameter_m * draw.diameter_m;
        let t = declared.flaw_density_per_m3
            * volume
            * libm::pow(
                draw.strength_pa / declared.weibull_sigma0_pa,
                declared.weibull_m,
            );
        sum_t += t;
        sum_t2 += t * t;
        sum_ln_t += libm::log(t);
    }
    if n == 0 {
        report.unresolved = 6 + fractions.len() as u32;
        return report;
    }
    let nf = n as f64;
    for (k, fraction) in fractions.iter().enumerate() {
        let tolerance = CERT_Z * libm::sqrt(fraction * (1.0 - fraction) / nf) + 1.0 / nf;
        let deviation = libm::fabs(counts[k] as f64 / nf - fraction);
        apply_check(
            &mut report,
            CERT_FLOOR_MEAN,
            deviation,
            tolerance,
            CertCheck::MineralFraction(k),
        );
    }
    apply_check(
        &mut report,
        CERT_FLOOR_MEAN,
        libm::fabs(sum_z / nf),
        CERT_Z * libm::sqrt(1.0 / nf),
        CertCheck::SizeMean,
    );
    apply_check(
        &mut report,
        CERT_FLOOR_SPREAD,
        libm::fabs(sum_z2 / nf - 1.0),
        CERT_Z * libm::sqrt(2.0 / nf),
        CertCheck::SizeSpread,
    );
    apply_check(
        &mut report,
        CERT_FLOOR_MEAN,
        libm::fabs(sum_ln_t / nf + EULER_GAMMA),
        CERT_Z * libm::sqrt(PI * PI / 6.0 / nf),
        CertCheck::StrengthLogMean,
    );
    apply_check(
        &mut report,
        CERT_FLOOR_MEAN,
        libm::fabs(sum_t / nf - 1.0),
        CERT_Z * libm::sqrt(1.0 / nf),
        CertCheck::StrengthMean,
    );
    apply_check(
        &mut report,
        CERT_FLOOR_SPREAD,
        libm::fabs(sum_t2 / nf - 2.0),
        CERT_Z * libm::sqrt(20.0 / nf),
        CertCheck::StrengthSpread,
    );
    report
}

// ---------------------------------------------------------------------------
// The materializer.
// ---------------------------------------------------------------------------

/// Everything a [`BoundarySelector`] may condition a child's boundary flag on: the
/// parent being decomposed, the child's ordinal within the batch, and the child's own
/// realized draw. Holons carry no spatial coordinates — any spatial tip-side test
/// lives in the caller's chart, keyed off parent identity and child ordinal.
#[derive(Clone, Copy, Debug)]
pub struct ChildBoundaryContext<'a> {
    pub parent: usize,
    pub parent_record: &'a RuntimeHolon,
    pub child_index: usize,
    pub fanout: usize,
    pub draw: &'a GrainDraw,
}

/// Per-child boundary selection — a VALUES-only extension point. The selector decides
/// which drawn children carry `boundary = true` and therefore which branches the
/// adaptive selector may descend. It must be a pure function of its context: it runs
/// after the draw and never touches the PRNG stream, so the quenched realization is
/// bit-identical whatever the selector says.
///
/// Why this exists: in `certify_runtime`, `GrainFloor` outranks
/// `RefinementUnavailable`, so a mixed frontier containing an active BOUNDARY grain-1
/// leaf halts adaptive materialization. With inherit-all boundary children a fanout
/// subtree therefore cannot descend to the grain floor; marking only tip-side
/// children lets `certify_runtime_adaptive` refine exactly the branch that matters
/// (the crack tip) while off-tip siblings stay coarse and latent.
pub trait BoundarySelector {
    fn child_boundary(&self, context: ChildBoundaryContext<'_>) -> bool;
}

impl<F: Fn(ChildBoundaryContext<'_>) -> bool> BoundarySelector for F {
    fn child_boundary(&self, context: ChildBoundaryContext<'_>) -> bool {
        self(context)
    }
}

/// The default [`BoundarySelector`]: every child inherits the parent's boundary flag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InheritParent;

impl BoundarySelector for InheritParent {
    fn child_boundary(&self, context: ChildBoundaryContext<'_>) -> bool {
        context.parent_record.is_boundary()
    }
}

/// Descriptor-as-generator [`RuntimeMaterializer`]: reads the subject's
/// [`MaterialBinding`], walks the descriptor subtree, and draws subject children
/// deterministically from the descriptor's declared distributions — mineral identity
/// per the modal fractions, sizes per the log-normal, quenched flaw strength per the
/// weakest-link Weibull — seeded from the seed persisted on the subject holon.
///
/// The binding's `descriptor_holon` indexes the borrowed descriptor arena; its
/// `subject_holon` indexes the scene arena passed to `materialize`. Only the subject
/// and its descendants are handled; other holons are declined with `Ok(false)` so
/// materializers for other bodies can coexist. Children's boundary flags come from
/// the [`BoundarySelector`] `B` (default: inherit the parent's).
pub struct DescriptorMaterializer<'d, B = InheritParent> {
    descriptors: &'d RuntimeArena,
    binding: MaterialBinding,
    fanout: usize,
    /// Whole-state slot of the ROOT subject's persisted seed. Children created here
    /// always carry theirs at [`GRAIN_SEED_HI`]/[`GRAIN_SEED_LO`].
    seed_slot: usize,
    view: StoneDescriptorView,
    boundary: B,
    /// Test-only mutant hook: when set, draws use this law while certification still
    /// runs against the descriptor's declared law. This is how the mutation tests
    /// plant a wrong-distribution generator without forking the code path.
    planted_law: Option<DrawParams>,
    last_report: Option<StatisticalReport>,
}

impl<'d> DescriptorMaterializer<'d, InheritParent> {
    pub fn new(
        descriptors: &'d RuntimeArena,
        binding: MaterialBinding,
        fanout: usize,
        seed_slot: usize,
    ) -> Result<Self, DescriptorError> {
        Self::with_boundary(descriptors, binding, fanout, seed_slot, InheritParent)
    }
}

impl<'d, B: BoundarySelector> DescriptorMaterializer<'d, B> {
    /// Like [`DescriptorMaterializer::new`], with a caller-supplied per-child
    /// [`BoundarySelector`] (any `Fn(ChildBoundaryContext<'_>) -> bool` works).
    pub fn with_boundary(
        descriptors: &'d RuntimeArena,
        binding: MaterialBinding,
        fanout: usize,
        seed_slot: usize,
        boundary: B,
    ) -> Result<Self, DescriptorError> {
        if fanout < 2 {
            return Err(DescriptorError::InvalidConfiguration);
        }
        let view = read_stone_descriptor(descriptors, binding.descriptor_holon)?;
        Ok(Self {
            descriptors,
            binding,
            fanout,
            seed_slot,
            view,
            boundary,
            planted_law: None,
            last_report: None,
        })
    }

    /// The declared law and modal fractions this generator draws from and certifies
    /// against.
    pub fn declared(&self) -> (&DrawParams, &[f64]) {
        (&self.view.params, &self.view.fractions)
    }

    pub const fn descriptors(&self) -> &'d RuntimeArena {
        self.descriptors
    }

    /// The statistical report of the most recent drawn batch, retained because the
    /// [`RuntimeMaterializer`] trait cannot carry it in its error type.
    pub const fn last_report(&self) -> Option<&StatisticalReport> {
        self.last_report.as_ref()
    }

    /// Full-fidelity materialization: like the trait method, but statistical
    /// rejection and missing-Record conditions keep their own error variants.
    pub fn materialize_described(
        &mut self,
        arena: &mut RuntimeArena,
        holon: usize,
    ) -> Result<bool, DescriptorError> {
        let record = *arena
            .holon(holon)
            .ok_or(DescriptorError::Holon(HolonError::InvalidParent))?;
        if record.decomposition != Decomposition::Latent || record.grain_units == 1 {
            return Ok(false);
        }
        if !arena.is_descendant_or_self(holon, self.binding.subject_holon) {
            return Ok(false);
        }
        let slot = if holon == self.binding.subject_holon {
            self.seed_slot
        } else {
            GRAIN_SEED_HI
        };
        let seed = arena
            .whole_state(holon)
            .and_then(|whole| whole.get(slot..slot + SEED_SLOTS))
            .and_then(decode_seed)
            .ok_or(DescriptorError::SeedMissing)?;

        let draw_law = self.planted_law.unwrap_or(self.view.params);
        let draws = draw_grains(seed, &draw_law, &self.view.fractions, self.fanout);
        let report = certify_grains(&draws, &self.view.params, &self.view.fractions);
        self.last_report = Some(report);
        if report.rejected() {
            return Err(DescriptorError::Statistics(report));
        }

        let grain_units = (record.grain_units / 2).max(1);
        let decomposition = if grain_units == 1 {
            Decomposition::Leaf
        } else {
            Decomposition::Latent
        };
        let depth = record
            .depth
            .checked_add(1)
            .ok_or(DescriptorError::Holon(HolonError::InvalidDepth))?;
        let parts = self.fanout as u64;
        let wholes: Vec<[f64; GRAIN_WHOLE_LEN]> =
            draws.iter().map(GrainDraw::to_whole).collect();
        let specs: Vec<RuntimeHolonSpec<'_>> = wholes
            .iter()
            .enumerate()
            .map(|(i, whole)| RuntimeHolonSpec {
                parent: holon as u32,
                depth,
                grain_units,
                gross: GrossState::aggregate(
                    share_u64(record.gross.constituents, parts, i as u64),
                    share_u64(record.gross.occupancy, parts, i as u64),
                    [
                        share_i64(record.gross.momentum[0], parts as i64, i as i64),
                        share_i64(record.gross.momentum[1], parts as i64, i as i64),
                    ],
                ),
                whole,
                channels: record.channels,
                boundary: self.boundary.child_boundary(ChildBoundaryContext {
                    parent: holon,
                    parent_record: &record,
                    child_index: i,
                    fanout: self.fanout,
                    draw: &draws[i],
                }),
                decomposition,
            })
            .collect();
        arena.materialize(holon, &specs)?;
        Ok(true)
    }
}

impl<B: BoundarySelector> RuntimeMaterializer for DescriptorMaterializer<'_, B> {
    fn materialize(&mut self, arena: &mut RuntimeArena, holon: usize) -> Result<bool, HolonError> {
        self.materialize_described(arena, holon).map_err(|error| match error {
            DescriptorError::Holon(inner) => inner,
            // The certificate IS the downward composition check: a batch whose
            // statistics do not compose to the declared law is rejected exactly like
            // a ledger that does not compose. The distinguishing report stays on
            // `last_report`.
            DescriptorError::Statistics(_) => HolonError::GrossStateDoesNotCompose,
            _ => HolonError::InvalidDecomposition,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::holon::{CertificationStatus, Evaluation};
    use crate::material::IsotropicMaterial;
    use crate::runtime::{certify_runtime_adaptive, RuntimeBoundaryModel, RuntimeFrontier};

    fn demo_law() -> DrawParams {
        DrawParams {
            grain_mu_ln_m: libm::log(7.5e-4),
            grain_sigma_ln: 0.5,
            weibull_m: 10.0,
            weibull_sigma0_pa: 2.0e8,
            flaw_density_per_m3: 1.0e10,
        }
    }

    const MODAL: [u64; 3] = [30, 60, 10];

    fn descriptor_arena() -> RuntimeArena {
        build_stone_descriptor(1_000_000, &MODAL, &demo_law()).unwrap()
    }

    fn wall_arena(seed: u64, grain: u32) -> RuntimeArena {
        let whole = encode_seed(seed);
        let specs = [RuntimeHolonSpec {
            parent: NO_RUNTIME_HOLON,
            depth: 0,
            grain_units: grain,
            gross: GrossState::aggregate(1_000_000, 2_000_000, [1_000_000, -3]),
            whole: &whole,
            channels: Channels::REG_PLUS.union(Channels::MECHANICAL),
            boundary: true,
            decomposition: Decomposition::Latent,
        }];
        RuntimeArena::from_specs(&specs, 0).unwrap()
    }

    fn binding() -> MaterialBinding {
        MaterialBinding {
            subject_holon: 0,
            descriptor_holon: 0,
            properties: IsotropicMaterial::DEMO_CALIBRATION,
        }
    }

    fn expand_fully<B: BoundarySelector>(
        materializer: &mut DescriptorMaterializer<'_, B>,
        arena: &mut RuntimeArena,
    ) {
        let mut i = 0;
        while i < arena.len() {
            if arena.holon(i).unwrap().decomposition == Decomposition::Latent {
                assert!(materializer.materialize_described(arena, i).unwrap());
            }
            i += 1;
        }
    }

    fn leaf_grains(arena: &RuntimeArena) -> Vec<GrainDraw> {
        (0..arena.len())
            .filter(|i| arena.holon(*i).unwrap().decomposition == Decomposition::Leaf)
            .map(|i| read_grain(arena, i).unwrap())
            .collect()
    }

    #[test]
    fn splitmix64_matches_the_reference_sequence() {
        // First outputs of the reference splitmix64.c at state 0.
        let mut stream = SplitMix64::new(0);
        assert_eq!(stream.next_u64(), 0xE220_A839_7B1D_CDAF);
        assert_eq!(stream.next_u64(), 0x6E78_9E6A_A1B9_65F4);
        assert_eq!(stream.next_u64(), 0x06C4_5D18_8009_454F);
    }

    #[test]
    fn seed_roundtrips_exactly_through_whole_state() {
        for seed in [0, 1, u64::MAX, 0xDEAD_BEEF_CAFE_F00D] {
            assert_eq!(decode_seed(&encode_seed(seed)), Some(seed));
        }
        assert_eq!(decode_seed(&[0.5, 0.0]), None);
        assert_eq!(decode_seed(&[-1.0, 0.0]), None);
    }

    // (a) LEDGER EXACTNESS -------------------------------------------------

    #[test]
    fn descriptor_modal_split_is_exact_integers() {
        let arena = descriptor_arena();
        let counts: Vec<u64> = (1..arena.len())
            .map(|i| arena.holon(i).unwrap().gross.constituents)
            .collect();
        assert_eq!(counts, vec![300_000, 600_000, 100_000]);
        arena.validate().unwrap();
        // An uneven total still composes exactly.
        let uneven = build_stone_descriptor(1_000_001, &[30, 60, 10], &demo_law()).unwrap();
        let sum: u64 = (1..uneven.len())
            .map(|i| uneven.holon(i).unwrap().gross.constituents)
            .sum();
        assert_eq!(sum, 1_000_001);
    }

    #[test]
    fn subject_children_compose_exactly_including_negative_momentum() {
        let descriptors = descriptor_arena();
        let mut materializer =
            DescriptorMaterializer::new(&descriptors, binding(), 7, 0).unwrap();
        let mut arena = wall_arena(42, 8);
        expand_fully(&mut materializer, &mut arena);
        arena.validate().unwrap();
        let root = arena.holon(0).unwrap().gross;
        let composed = (0..arena.len())
            .filter(|i| arena.holon(*i).unwrap().parent == 0)
            .fold(GrossState::ZERO, |acc, i| {
                acc.combine(arena.holon(i).unwrap().gross)
            });
        assert_eq!(composed, root);
    }

    #[test]
    fn ledger_mutant_one_moved_constituent_is_rejected() {
        // MUTATION (a): tamper the exact split by one constituent; the existing
        // conservation gate must fire and commit nothing.
        let mut arena = wall_arena(42, 8);
        let record = *arena.holon(0).unwrap();
        let before = arena.clone();
        let whole = [0.0; GRAIN_WHOLE_LEN];
        let specs: Vec<RuntimeHolonSpec<'_>> = (0..4_u64)
            .map(|i| RuntimeHolonSpec {
                parent: 0,
                depth: 1,
                grain_units: 4,
                gross: GrossState::aggregate(
                    share_u64(record.gross.constituents, 4, i) + u64::from(i == 0),
                    share_u64(record.gross.occupancy, 4, i),
                    [
                        share_i64(record.gross.momentum[0], 4, i as i64),
                        share_i64(record.gross.momentum[1], 4, i as i64),
                    ],
                ),
                whole: &whole,
                channels: record.channels,
                boundary: true,
                decomposition: Decomposition::Latent,
            })
            .collect();
        assert_eq!(
            arena.materialize(0, &specs),
            Err(HolonError::GrossStateDoesNotCompose)
        );
        assert_eq!(arena, before);
    }

    // (b) REPLAY DETERMINISM ----------------------------------------------

    #[test]
    fn same_wall_decomposed_twice_is_bit_identical() {
        let descriptors = descriptor_arena();
        let run = |seed: u64| {
            let mut materializer =
                DescriptorMaterializer::new(&descriptors, binding(), 4, 0).unwrap();
            let mut arena = wall_arena(seed, 8);
            expand_fully(&mut materializer, &mut arena);
            arena
        };
        let first = run(1234);
        let second = run(1234);
        assert_eq!(first.holons(), second.holons());
        let first_bits: Vec<u64> = first.whole_scalars().iter().map(|x| x.to_bits()).collect();
        let second_bits: Vec<u64> = second.whole_scalars().iter().map(|x| x.to_bits()).collect();
        assert_eq!(first_bits, second_bits);

        // MUTATION (b): a different persisted seed must change the realization —
        // a generator that ignores the Record seed would pass the identity check
        // above vacuously.
        let other = run(1235);
        let other_bits: Vec<u64> = other.whole_scalars().iter().map(|x| x.to_bits()).collect();
        assert_ne!(first_bits, other_bits);
    }

    #[test]
    fn missing_record_seed_is_refused_not_invented() {
        let descriptors = descriptor_arena();
        let mut materializer =
            DescriptorMaterializer::new(&descriptors, binding(), 4, 0).unwrap();
        let specs = [RuntimeHolonSpec {
            parent: NO_RUNTIME_HOLON,
            depth: 0,
            grain_units: 8,
            gross: GrossState::aggregate(1_000, 2_000, [0, 0]),
            whole: &[],
            channels: Channels::REG_PLUS,
            boundary: true,
            decomposition: Decomposition::Latent,
        }];
        let mut arena = RuntimeArena::from_specs(&specs, 0).unwrap();
        assert_eq!(
            materializer.materialize_described(&mut arena, 0),
            Err(DescriptorError::SeedMissing)
        );
    }

    // (c) THE CERTIFICATE FIRES ON PLANTED WRONG DISTRIBUTIONS -------------

    #[test]
    fn certificate_fires_on_wrong_weibull_modulus() {
        let declared = demo_law();
        let mut planted = declared;
        planted.weibull_m = declared.weibull_m * 3.8;
        let fractions = [0.3, 0.6, 0.1];
        let draws = draw_grains(7, &planted, &fractions, 512);
        let report = certify_grains(&draws, &declared, &fractions);
        assert!(
            matches!(
                report.failed,
                Some(
                    CertCheck::StrengthLogMean
                        | CertCheck::StrengthMean
                        | CertCheck::StrengthSpread
                )
            ),
            "{report:?}"
        );
    }

    #[test]
    fn certificate_fires_on_wrong_weibull_scale() {
        let declared = demo_law();
        let mut planted = declared;
        planted.weibull_sigma0_pa = declared.weibull_sigma0_pa * 1.5;
        let fractions = [0.3, 0.6, 0.1];
        let draws = draw_grains(11, &planted, &fractions, 512);
        let report = certify_grains(&draws, &declared, &fractions);
        assert!(
            matches!(
                report.failed,
                Some(
                    CertCheck::StrengthLogMean
                        | CertCheck::StrengthMean
                        | CertCheck::StrengthSpread
                )
            ),
            "{report:?}"
        );
    }

    #[test]
    fn certificate_fires_on_wrong_grain_size_spread() {
        let declared = demo_law();
        let mut planted = declared;
        planted.grain_sigma_ln = declared.grain_sigma_ln * 2.0;
        let fractions = [0.3, 0.6, 0.1];
        let draws = draw_grains(13, &planted, &fractions, 512);
        let report = certify_grains(&draws, &declared, &fractions);
        assert!(
            matches!(
                report.failed,
                Some(CertCheck::SizeSpread | CertCheck::StrengthLogMean | CertCheck::StrengthMean)
            ),
            "{report:?}"
        );
    }

    #[test]
    fn certificate_fires_on_wrong_modal_fractions() {
        let declared = demo_law();
        let planted_fractions = [0.6, 0.3, 0.1];
        let declared_fractions = [0.3, 0.6, 0.1];
        let draws = draw_grains(17, &declared, &planted_fractions, 512);
        let report = certify_grains(&draws, &declared, &declared_fractions);
        assert!(
            matches!(report.failed, Some(CertCheck::MineralFraction(_))),
            "{report:?}"
        );
    }

    #[test]
    fn trait_path_rejects_a_planted_mutant_like_a_ledger_violation() {
        // MUTATION (c), end to end: the generator draws from a wrong Weibull while
        // the certificate checks the descriptor's declared law; the trait surfaces
        // the rejection as a composition failure and commits NOTHING.
        let descriptors = descriptor_arena();
        let mut materializer =
            DescriptorMaterializer::new(&descriptors, binding(), 512, 0).unwrap();
        let mut mutant_law = materializer.view.params;
        mutant_law.weibull_m *= 3.8;
        materializer.planted_law = Some(mutant_law);
        let mut arena = wall_arena(42, 8);
        let before = arena.clone();
        assert_eq!(
            RuntimeMaterializer::materialize(&mut materializer, &mut arena, 0),
            Err(HolonError::GrossStateDoesNotCompose)
        );
        assert_eq!(arena, before);
        assert!(materializer.last_report().unwrap().rejected());
    }

    // (d) THE CERTIFICATE DOES NOT FIRE WHERE IT CANNOT RESOLVE ------------

    #[test]
    fn certificate_is_unresolved_not_fired_below_its_floors() {
        let declared = demo_law();
        let mut planted = declared;
        planted.weibull_m = declared.weibull_m * 3.8;
        let fractions = [0.3, 0.6, 0.1];
        let draws = draw_grains(7, &planted, &fractions, 6);
        let report = certify_grains(&draws, &declared, &fractions);
        assert!(!report.rejected(), "{report:?}");
        // Every distributional check is below its floor at n = 6; nothing passed
        // by discrimination, everything by declared lack of resolution.
        assert_eq!(report.passed, 0);
        assert!(report.unresolved >= 6, "{report:?}");
    }

    #[test]
    fn honest_generator_passes_at_large_n_and_across_many_seeds() {
        let declared = demo_law();
        let fractions = [0.3, 0.6, 0.1];
        let big = draw_grains(99, &declared, &fractions, 4096);
        let report = certify_grains(&big, &declared, &fractions);
        assert!(!report.rejected(), "{report:?}");
        assert_eq!(report.unresolved, 0);
        // Deterministic false-fire scan: 200 honest batches at n = 256.
        for seed in 0..200_u64 {
            let draws = draw_grains(seed, &declared, &fractions, 256);
            let report = certify_grains(&draws, &declared, &fractions);
            assert!(!report.rejected(), "seed {seed}: {report:?}");
        }
    }

    // INTEGRATION ----------------------------------------------------------

    struct GrainFloorModel;

    impl RuntimeBoundaryModel<1> for GrainFloorModel {
        fn evaluate(&mut self, arena: &RuntimeArena, frontier: &RuntimeFrontier) -> Evaluation<1> {
            let grain = frontier.represented_grain(arena, 0) as f64;
            Evaluation {
                observables: [grain],
                macro_error_bound: grain * grain * 1.0e-4,
                conservation_residual: 0.0,
            }
        }

        fn refinement_priority(
            &self,
            arena: &RuntimeArena,
            _frontier: &RuntimeFrontier,
            holon: usize,
        ) -> f64 {
            arena.holons()[holon].grain_units as f64
        }
    }

    #[test]
    fn adaptive_certification_materializes_through_the_descriptor() {
        // Root grain 8, fanout 4, tolerance passing at represented grain 2: the
        // selector must materialize the root and then all four grain-4 children (five
        // materializations) before the sixteen grain-2 grandchildren certify. The
        // tolerance deliberately stops ABOVE the grain floor: with inherited boundary
        // flags a mixed frontier containing grain-1 leaves reads GrainFloor, which
        // outranks RefinementUnavailable and halts adaptive descent — the engine
        // interaction a per-child BoundarySelector exists to steer around (see
        // `selective_boundary_descends_past_the_inherit_all_halt_point`).
        let descriptors = descriptor_arena();
        let mut materializer =
            DescriptorMaterializer::new(&descriptors, binding(), 4, 0).unwrap();
        let mut arena = wall_arena(42, 8);
        let result = certify_runtime_adaptive(
            &mut arena,
            &mut GrainFloorModel,
            &mut materializer,
            4.0e-4,
            1.0e-12,
        )
        .unwrap();
        assert!(result.certificate.passed(), "{:?}", result.certificate.status);
        assert_eq!(result.materializations, 5);
        assert_eq!(result.certificate.frontier.represented_grain(&arena, 0), 2);
        arena.validate().unwrap();
        let grains: Vec<GrainDraw> = (0..arena.len())
            .filter(|i| arena.holon(*i).unwrap().depth == 2)
            .map(|i| read_grain(&arena, i).unwrap())
            .collect();
        assert_eq!(grains.len(), 16);
        for grain in &grains {
            assert!(grain.mineral < 3);
            assert!(grain.diameter_m > 0.0 && grain.diameter_m.is_finite());
            assert!(grain.strength_pa > 0.0 && grain.strength_pa.is_finite());
        }
    }

    #[test]
    fn selective_boundary_descends_past_the_inherit_all_halt_point() {
        let descriptors = descriptor_arena();
        // CONTROL (the mutant that proves this test can fail): inherit-all boundary
        // at a tolerance requiring grain 1 halts at GrainFloor on the first mixed
        // frontier containing boundary leaves — the previous halt point.
        let mut inherit = DescriptorMaterializer::new(&descriptors, binding(), 4, 0).unwrap();
        let mut arena = wall_arena(42, 8);
        let halted = certify_runtime_adaptive(
            &mut arena,
            &mut GrainFloorModel,
            &mut inherit,
            1.0e-4,
            1.0e-12,
        )
        .unwrap();
        assert_eq!(halted.certificate.status, CertificationStatus::GrainFloor);

        // Tip-side-only children carry the flag: the same scene, model, and tolerance
        // now certify at the grain floor, materializing exactly the boundary chain
        // (root -> one grain-4 child -> one grain-2 child) while off-tip siblings
        // stay coarse and latent.
        let tip_side = |context: ChildBoundaryContext<'_>| {
            context.parent_record.is_boundary() && context.child_index == 0
        };
        let mut selective =
            DescriptorMaterializer::with_boundary(&descriptors, binding(), 4, 0, tip_side)
                .unwrap();
        let mut arena = wall_arena(42, 8);
        let result = certify_runtime_adaptive(
            &mut arena,
            &mut GrainFloorModel,
            &mut selective,
            1.0e-4,
            1.0e-12,
        )
        .unwrap();
        assert!(result.certificate.passed(), "{:?}", result.certificate.status);
        assert_eq!(result.materializations, 3);
        assert_eq!(result.certificate.frontier.represented_grain(&arena, 0), 1);
        assert_eq!(arena.len(), 13);
        arena.validate().unwrap();
    }

    #[test]
    fn boundary_selector_sees_the_true_draw_and_never_perturbs_it() {
        let descriptors = descriptor_arena();
        // MUTATION: a selector keyed on the child's realized draw. If the
        // materializer handed the selector a wrong draw or ordinal, the stored
        // boundary flags would disagree with the read-back grains below.
        let quartz_only = |context: ChildBoundaryContext<'_>| context.draw.mineral == 0;
        let mut selective =
            DescriptorMaterializer::with_boundary(&descriptors, binding(), 8, 0, quartz_only)
                .unwrap();
        let mut arena = wall_arena(42, 8);
        expand_fully(&mut selective, &mut arena);
        let mut boundary_count = 0;
        for i in 1..arena.len() {
            let grain = read_grain(&arena, i).unwrap();
            assert_eq!(arena.holon(i).unwrap().is_boundary(), grain.mineral == 0);
            boundary_count += usize::from(grain.mineral == 0);
        }
        assert!(boundary_count > 0 && boundary_count < arena.len() - 1);

        // The selector runs after the draw and outside the PRNG stream: the quenched
        // realization must be bit-identical to the inherit-parent run. A selector
        // implementation that consumed or reordered draws would fail here.
        let mut inherit = DescriptorMaterializer::new(&descriptors, binding(), 8, 0).unwrap();
        let mut control = wall_arena(42, 8);
        expand_fully(&mut inherit, &mut control);
        let selective_bits: Vec<u64> =
            arena.whole_scalars().iter().map(|x| x.to_bits()).collect();
        let control_bits: Vec<u64> =
            control.whole_scalars().iter().map(|x| x.to_bits()).collect();
        assert_eq!(selective_bits, control_bits);
    }

    #[test]
    fn materialized_ensemble_certifies_against_the_descriptor() {
        let descriptors = descriptor_arena();
        let mut materializer =
            DescriptorMaterializer::new(&descriptors, binding(), 8, 0).unwrap();
        let mut arena = wall_arena(42, 8);
        expand_fully(&mut materializer, &mut arena);
        let grains = leaf_grains(&arena);
        assert_eq!(grains.len(), 512);
        let (declared, fractions) = materializer.declared();
        let report = certify_grains(&grains, declared, fractions);
        assert!(!report.rejected(), "{report:?}");
        assert_eq!(report.unresolved, 0);
    }

    #[test]
    fn weighted_shares_are_exact_for_adversarial_totals() {
        for total in [-7_i128, 0, 1, 999_983, 1_000_000] {
            let shares = weighted_shares(total, &[30, 60, 10]);
            assert_eq!(shares.iter().sum::<i128>(), total);
        }
    }
}
