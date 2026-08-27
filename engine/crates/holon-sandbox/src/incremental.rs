//! Incremental certification: the same selector, without the restart.
//!
//! # Why this exists (measured, not assumed)
//!
//! `ciris_sim_core::runtime::certify_runtime_adaptive` restarts selection at the
//! encounter root after every materialization, and `certify_runtime_in` refines exactly
//! one holon per `O(active)` evaluation. On this workstation (release, x86_64), with a
//! boundary model that does no physics at all — so the numbers are ENGINE cost only:
//!
//! | materializations | resident holons | seconds |
//! |---:|---:|---:|
//! | 22 | 89 | 0.0001 |
//! | 86 | 345 | 0.0023 |
//! | 342 | 1369 | 0.1286 |
//! | 1366 | 5465 | 6.9428 |
//!
//! Four times the materializations costs fifty-four times the wall clock: the growth is
//! cubic. A single full descent in isolation is quadratic (1369 holons 0.75 ms; 5465
//! holons 12.20 ms — already over a 60 fps frame by itself). A browser scene wants
//! thousands of resident cells per throw, so the shipped entry point is three orders of
//! magnitude short of the budget. This module pays that debt in-crate.
//!
//! # What is NOT changed
//!
//! The mathematics, the greedy rule, the tie-break, and the three verdicts are the
//! shipped ones. This is a workspace and a bookkeeping change, never a semantics
//! change:
//!
//! * the frontier still begins at the root and only ever refines;
//! * the candidate is still the highest-priority ACTIVE, BOUNDARY holon, with ties
//!   broken toward the LOWEST holon id — matching `certify_runtime_in`'s strict
//!   `score > priority` over ascending `active_indices()`;
//! * `GrainFloor` still outranks `RefinementUnavailable`;
//! * materialization is still transactional through `RuntimeArena::materialize`, so the
//!   REG+ ledger check is the core's, untouched.
//!
//! # The precondition, stated rather than hidden
//!
//! Dropping the restart is only sound for a **cell-max-decomposable** model: one whose
//! frontier error bound is the maximum of a per-cell error over the active set, and
//! whose refinement priority ranks by that same per-cell error. [`CellwiseModel`] is
//! that contract, and it is not a narrowing invented for convenience — the shipped
//! `FractureModel` already has exactly this shape (`fracture.rs`'s `bound()` is
//! `max(size / allowed_spacing(distance))` over `active_indices()`, and its
//! `refinement_priority` ranks by the same ratio).
//!
//! # Why the restart can be dropped, and the one case where it cannot
//!
//! For a model whose cell error is a pure function of the arena and the holon, the
//! greedy descent is memoryless: at every step it refines the highest-error refinable
//! active cell, so the sequence of frontiers is a function of the arena alone.
//! Materialization only APPENDS children, and a newly appended child is not active
//! until its parent is refined — so a restarted descent retraces exactly the frontier
//! it left and then continues. The restart is confluent, and skipping it is free.
//!
//! That argument uses staticness, and some models are not static: the shipped fracture
//! model's damage surface MOVES when a cohesive solve runs, and every cell error is
//! measured from that surface. Once the field moves, the frontier the incremental path
//! is standing on is path-dependent — refinements already made were justified under the
//! old field — and a restarted descent can legitimately reach a different frontier.
//!
//! So the restart is not dropped; it is made CONDITIONAL, and the condition is the
//! model's own declaration. [`CellwiseModel::epoch`] identifies the current cell-error
//! field. After a materialization, if the epoch has moved since the last descent began,
//! this certifier resets to the root and re-descends exactly as
//! `certify_runtime_adaptive` does. A static-field model never pays that; a moving-field
//! model pays it precisely where the shipped certifier does, and stays equivalent.
//!
//! This was not reasoned out in advance. The equivalence test below disagreed on a
//! moving-field model, and tracing that disagreement is what produced the paragraph
//! above. The static-field speedup is the same either way; the honest scope of the
//! claim is narrower than it first looked.
//!
//! The precondition is enforced by test, not by trust: [`certify_incremental`] and
//! `certify_runtime_adaptive` are run over the SAME model through [`PriorityAdapter`],
//! and their status, observables, residual, bound, materialization count and active
//! frontier must all agree bit-for-bit — with a moving-field model among the cases.

use std::cmp::Ordering;
use std::collections::BinaryHeap;

use ciris_sim_core::holon::{CertificationStatus, Evaluation, HolonError};
use ciris_sim_core::runtime::{
    RuntimeArena, RuntimeBoundaryModel, RuntimeFrontier, RuntimeMaterializer,
    NO_RUNTIME_HOLON,
};

/// Sentinel for "no parent", named locally so the child index reads clearly.
const NO_PARENT: u32 = NO_RUNTIME_HOLON;

/// What one settled frontier reads out. Separated from the per-cell error because the
/// error gates whether a settle is worth running at all — the shipped fracture model
/// runs its cohesive solve only on frontiers that already pass the resolution
/// surrogate, and this keeps that ordering.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settled<const O: usize> {
    pub observables: [f64; O],
    pub conservation_residual: f64,
}

/// A realization whose frontier error bound is the MAXIMUM of a per-cell error.
///
/// The two obligations are the whole precondition of [`certify_incremental`]:
///
/// 1. `macro_error_bound(frontier) == max over active h of cell_error(h)`;
/// 2. refinement ranks by that same `cell_error`.
///
/// Anything a model needs that is not cell-local goes through [`Self::epoch`].
pub trait CellwiseModel<const O: usize> {
    /// This cell's contribution to the frontier's error bound. Must be finite and
    /// non-negative; larger means "refine me first".
    fn cell_error(&mut self, arena: &RuntimeArena, holon: usize) -> f64;

    /// Read out the frontier once its bound has passed the macro tolerance. May move
    /// model state (running a solve, relocating a damage surface); if it does, it must
    /// bump [`Self::epoch`], and every cached cell error is then discarded and the
    /// bound recomputed before the certificate can be issued.
    fn settle(&mut self, arena: &RuntimeArena, active: &[usize], bound: f64) -> Settled<O>;

    /// Monotone counter identifying the model's current cell-error field. A model whose
    /// cell errors are a pure function of the arena and the holon never changes it.
    fn epoch(&self) -> u64 {
        0
    }
}

/// The certificate this module returns. Field-for-field the shipped
/// `RuntimeResolutionCertificate` plus the materialization count, so the two paths can
/// be compared without translation.
#[derive(Clone, Debug, PartialEq)]
pub struct IncrementalCertificate<const O: usize> {
    pub status: CertificationStatus,
    pub observables: [f64; O],
    pub macro_error_bound: f64,
    pub conservation_residual: f64,
    /// Rounds of the greedy loop, the analogue of the shipped `evaluations` count.
    pub rounds: usize,
    pub materializations: usize,
    /// Resident holons active on the returned frontier.
    pub active: Vec<usize>,
}

impl<const O: usize> IncrementalCertificate<O> {
    pub const fn passed(&self) -> bool {
        matches!(self.status, CertificationStatus::Certified)
    }
}

/// Deliberate defects, plantable without forking the code path — the `ResidualMode`
/// pattern of `ciris_sim_core::fracture`. A gate that cannot fail proves nothing, and
/// the gate here is the bit-for-bit equivalence against the shipped certifier. Each
/// variant names one property that equivalence depends on; `tests::mutation` requires
/// every one of them to be CAUGHT.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mutation {
    #[default]
    None,
    /// MUTANT: break ties toward the HIGHEST holon id. `certify_runtime_in` scans
    /// ascending indices and selects on a strict `score > priority`, so the lowest id
    /// wins a tie; flipping it silently refines a different cell whenever two cells are
    /// equally bad, which on a symmetric scene is most of them.
    HighestIdWinsTies,
    /// MUTANT: treat an already-satisfied cell as a refinement candidate. Over-refines,
    /// so the frontier certifies at a finer grain than the coarsest valid one — the
    /// certificate still reads `Certified` and the observables are still finite, which
    /// is what makes this one worth planting.
    ZeroErrorRefines,
    /// MUTANT: materialize even while an active boundary holon sits at the grain floor,
    /// dropping the `GrainFloor` > `RefinementUnavailable` ranking. Turns an honest
    /// refusal into unbounded growth.
    IgnoreGrainFloorRank,
    /// MUTANT: skip the restart after a materialization that followed a field move.
    /// Only a moving-field model can detect this one.
    SkipEpochRestart,
    /// MUTANT: settle the frontier at a dead end. Reports observables measured on a mesh
    /// the certificate is simultaneously refusing to vouch for.
    SettleAtDeadEnd,
}

/// Heap entry. Ordering is (error ascending, tie key DESCENDING) so that
/// `BinaryHeap`'s max is the largest error and, among equal errors, the lowest holon
/// id — the tie-break `certify_runtime_in` produces by scanning ascending indices with
/// a strict `>`. The key is stored rather than derived so that
/// [`Mutation::HighestIdWinsTies`] can invert it without a second `Ord`.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Entry {
    error: f64,
    holon: usize,
    tie_key: usize,
    stamp: u64,
}

impl Eq for Entry {}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.error
            .total_cmp(&other.error)
            .then_with(|| other.tie_key.cmp(&self.tie_key))
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Which candidate class a heap holds. The shipped certifier draws refinement
/// candidates from active boundary `Expanded` holons and materialization candidates
/// from active boundary `Latent` ones; keeping them in separate heaps is what makes
/// both selections `O(log n)` instead of `O(n)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Class {
    Bound,
    Expandable,
    Materializable,
}

/// Reusable certification workspace. Allocation happens on growth only, so repeated
/// certifications of a growing scene do not re-allocate from scratch.
#[derive(Debug, Default)]
pub struct Workspace {
    active: Vec<bool>,
    stamp: Vec<u64>,
    error: Vec<f64>,
    bound: BinaryHeap<Entry>,
    expandable: BinaryHeap<Entry>,
    materializable: BinaryHeap<Entry>,
    active_list: Vec<usize>,
    /// First child and child count per holon, so refining an already-expanded holon is
    /// `O(children)` instead of a scan of the whole arena.
    ///
    /// `RuntimeArena::materialize` appends a parent's children contiguously, so a range
    /// is enough — and where it is NOT enough (a scene assembled by a builder rather
    /// than grown), `index_children` detects the discontiguity and leaves the range
    /// empty so the caller falls back to scanning. Assuming contiguity without checking
    /// it would silently drop children from the frontier, which is the worst failure
    /// this structure could have.
    children: Vec<(u32, u32)>,
    contiguous: bool,
    /// Holons whose materializer declined: still active, still contributing to the
    /// bound, no longer candidates.
    terminal: Vec<bool>,
    /// Active boundary holons already at the grain floor. Non-zero is what makes a
    /// dead end read `GrainFloor` rather than `RefinementUnavailable`.
    at_floor: usize,
    mutation: Mutation,
}

impl Workspace {
    pub fn new() -> Self {
        Self::default()
    }

    /// Plant a deliberate defect. Production callers never touch this; it exists so the
    /// equivalence gate can be shown to have teeth.
    pub fn with_mutation(mutation: Mutation) -> Self {
        Self {
            mutation,
            ..Self::default()
        }
    }

    fn grow(&mut self, len: usize) {
        if self.active.len() < len {
            self.active.resize(len, false);
            self.stamp.resize(len, 0);
            self.error.resize(len, 0.0);
            self.terminal.resize(len, false);
        }
    }

    /// Record each holon's children as a contiguous range, in one pass.
    ///
    /// This exists because of a comment that stopped being true. `refine` used to scan
    /// the whole arena for children, justified by "it runs only on pre-built expansions,
    /// never on the materialization path" — correct while a throw always began from a
    /// one-holon arena. Pinning the scene at observer acuity means the resting scene is
    /// already grown when a throw arrives, so every refinement took that scan and the
    /// descent went quadratic: 8.4 seconds for one sandbox throw against a 120 ms budget.
    fn index_children(&mut self, arena: &RuntimeArena) {
        self.children.clear();
        self.children.resize(arena.len(), (0, 0));
        self.contiguous = true;
        for (id, holon) in arena.holons().iter().enumerate() {
            let parent = holon.parent;
            if parent == crate::incremental::NO_PARENT {
                continue;
            }
            let parent = parent as usize;
            if parent >= self.children.len() {
                self.contiguous = false;
                return;
            }
            let (first, count) = self.children[parent];
            if count == 0 {
                self.children[parent] = (id as u32, 1);
            } else if first + count == id as u32 {
                self.children[parent] = (first, count + 1);
            } else {
                // Children are not contiguous, so a range cannot describe them. Say so
                // rather than losing one.
                self.contiguous = false;
                return;
            }
        }
    }

    fn reset(&mut self, arena: &RuntimeArena) {
        self.grow(arena.len());
        self.index_children(arena);
        self.active.iter_mut().for_each(|a| *a = false);
        self.stamp.iter_mut().for_each(|s| *s = 0);
        self.terminal.iter_mut().for_each(|t| *t = false);
        self.bound.clear();
        self.expandable.clear();
        self.materializable.clear();
        self.at_floor = 0;
    }

    fn is_stale(&self, entry: &Entry, class: Class) -> bool {
        if !self.active[entry.holon] || self.stamp[entry.holon] != entry.stamp {
            return true;
        }
        // A holon whose materializer declined is no longer a candidate, though it stays
        // active and keeps contributing to the bound.
        match class {
            Class::Bound => false,
            Class::Expandable | Class::Materializable => self.terminal[entry.holon],
        }
    }
}

/// Errors this certifier can return that the core's cannot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IncrementalError {
    /// A model returned a non-finite or negative cell error, breaking the
    /// [`CellwiseModel`] contract before any certificate could be formed.
    CellErrorNotFinite,
    /// Ran longer than the declared round budget. A browser event has a wall-clock
    /// budget; exceeding it is reported, never silently truncated into a verdict.
    RoundBudgetExhausted,
    Holon(HolonError),
}

impl From<HolonError> for IncrementalError {
    fn from(error: HolonError) -> Self {
        Self::Holon(error)
    }
}

/// Declared limits of one certification event.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Budget {
    pub macro_tolerance: f64,
    pub conservation_tolerance: f64,
    /// Hard cap on greedy rounds. The browser demo declares this so a throw cannot
    /// stall a frame loop; hitting it is an error, not a verdict.
    pub max_rounds: usize,
    /// Hard cap on resident holons, likewise declared rather than discovered.
    pub max_holons: usize,
}

impl Budget {
    pub const fn new(macro_tolerance: f64, conservation_tolerance: f64) -> Self {
        Self {
            macro_tolerance,
            conservation_tolerance,
            max_rounds: usize::MAX,
            max_holons: usize::MAX,
        }
    }
}

/// Certify while materializing only unresolved boundary branches, without restarting
/// selection at the root.
///
/// The frontier survives across materializations, and each refinement touches only the
/// holon being replaced and its children, so the descent is `O(n log n)` in resident
/// holons plus `O(epochs * n)` for whatever the model chooses to invalidate.
pub fn certify_incremental<const O: usize>(
    arena: &mut RuntimeArena,
    model: &mut impl CellwiseModel<O>,
    materializer: &mut impl RuntimeMaterializer,
    workspace: &mut Workspace,
    budget: Budget,
) -> Result<IncrementalCertificate<O>, IncrementalError> {
    workspace.reset(arena);
    let root = arena.root() as usize;
    let mut epoch = model.epoch();
    // The epoch this descent started under. If the model's field has moved since, the
    // frontier is path-dependent and the restart must be paid; see the module header.
    let mut descent_epoch = epoch;
    let mut materializations = 0_usize;
    let mut rounds = 0_usize;
    // The readout of the most recent frontier that was worth settling. A frontier whose
    // bound fails the macro tolerance is not solved at all, so its certificate carries
    // the last observables it had — the shipped `FractureModel::last_observables()`
    // behaviour, reproduced rather than reinvented.
    let mut last: Option<Settled<O>> = None;

    activate(arena, model, workspace, root)?;

    loop {
        rounds += 1;
        if rounds > budget.max_rounds {
            return Err(IncrementalError::RoundBudgetExhausted);
        }

        let bound = match peek(workspace, Class::Bound) {
            Some(entry) => entry.error,
            // An empty frontier cannot happen: the root is activated above and refine
            // always replaces one active holon with at least one child.
            None => 0.0,
        };

        if bound <= budget.macro_tolerance {
            collect_active(workspace);
            let settled = model.settle(arena, &workspace.active_list, bound);
            last = Some(settled);
            if model.epoch() != epoch {
                epoch = model.epoch();
                requench(arena, model, workspace)?;
                continue;
            }
            if settled.conservation_residual <= budget.conservation_tolerance {
                return Ok(finish(
                    workspace,
                    CertificationStatus::Certified,
                    settled,
                    bound,
                    rounds,
                    materializations,
                ));
            }
        }

        // Refine resident branches to exhaustion first. This is not a preference, it is
        // what `certify_runtime_adaptive` does: a full `certify_runtime` descent runs
        // to a dead end before any materialization is even considered.
        if let Some(entry) = pop_live(workspace, Class::Expandable) {
            refine(arena, model, workspace, entry.holon)?;
            continue;
        }

        // The descent is exhausted. `GrainFloor` OUTRANKS `RefinementUnavailable` in
        // the shipped certifier, and `certify_runtime_adaptive` materializes ONLY on
        // `RefinementUnavailable` — so an active boundary holon sitting at the grain
        // floor stops the whole adaptive process here. Reproducing that ranking is why
        // `at_floor` is tracked rather than derived at the end: getting it wrong turns
        // an honest refusal into an unbounded materialization loop.
        if workspace.at_floor > 0 && workspace.mutation != Mutation::IgnoreGrainFloorRank {
            return Ok(dead_end(
                arena,
                model,
                workspace,
                CertificationStatus::GrainFloor,
                last,
                bound,
                rounds,
                materializations,
            ));
        }

        if let Some(entry) = pop_live(workspace, Class::Materializable) {
            if arena.len() >= budget.max_holons {
                return Ok(dead_end(
                    arena,
                    model,
                    workspace,
                    CertificationStatus::RefinementUnavailable,
                    last,
                    bound,
                    rounds,
                    materializations,
                ));
            }
            let before = arena.len();
            if materializer.materialize(arena, entry.holon)? {
                materializations += 1;
                workspace.grow(arena.len());
                workspace.children.resize(arena.len(), (0, 0));
                if workspace.contiguous {
                    workspace.children[entry.holon] =
                        (before as u32, (arena.len() - before) as u32);
                }
                if descent_epoch != model.epoch()
                    && workspace.mutation != Mutation::SkipEpochRestart
                {
                    // The field moved during this descent, so the frontier below us is
                    // path-dependent. Pay the restart exactly where the shipped
                    // certifier pays it.
                    descent_epoch = model.epoch();
                    epoch = descent_epoch;
                    workspace.reset(arena);
                    activate(arena, model, workspace, root)?;
                    continue;
                }
                // `materialize` appends children contiguously, so the new ids are
                // exactly `before..arena.len()` and no scan of the arena is needed.
                deactivate(workspace, entry.holon);
                for child in before..arena.len() {
                    activate(arena, model, workspace, child)?;
                }
                continue;
            }
            // A materializer that declines leaves the branch unrefinable; it keeps
            // contributing to the bound and stops being a candidate.
            deactivate_as_terminal(workspace, entry.holon);
            continue;
        }

        return Ok(dead_end(
            arena,
            model,
            workspace,
            CertificationStatus::RefinementUnavailable,
            last,
            bound,
            rounds,
            materializations,
        ));
    }
}

/// A frontier that cannot be refined further reads out whatever the last SETTLED
/// frontier read, never a fresh settle: settling an unresolved frontier would report
/// numbers off a mesh the certificate is simultaneously refusing to vouch for.
fn dead_end<const O: usize>(
    arena: &RuntimeArena,
    model: &mut impl CellwiseModel<O>,
    workspace: &mut Workspace,
    status: CertificationStatus,
    last: Option<Settled<O>>,
    bound: f64,
    rounds: usize,
    materializations: usize,
) -> IncrementalCertificate<O> {
    let settled = if workspace.mutation == Mutation::SettleAtDeadEnd {
        collect_active(workspace);
        model.settle(arena, &workspace.active_list, bound)
    } else {
        last.unwrap_or(Settled {
            observables: [0.0; O],
            conservation_residual: 0.0,
        })
    };
    finish(workspace, status, settled, bound, rounds, materializations)
}

fn finish<const O: usize>(
    workspace: &mut Workspace,
    status: CertificationStatus,
    settled: Settled<O>,
    bound: f64,
    rounds: usize,
    materializations: usize,
) -> IncrementalCertificate<O> {
    collect_active(workspace);
    IncrementalCertificate {
        status,
        observables: settled.observables,
        macro_error_bound: bound,
        conservation_residual: settled.conservation_residual,
        rounds,
        materializations,
        active: workspace.active_list.clone(),
    }
}

fn collect_active(workspace: &mut Workspace) {
    workspace.active_list.clear();
    for (holon, active) in workspace.active.iter().enumerate() {
        if *active {
            workspace.active_list.push(holon);
        }
    }
}

fn heap_mut(workspace: &mut Workspace, class: Class) -> &mut BinaryHeap<Entry> {
    match class {
        Class::Bound => &mut workspace.bound,
        Class::Expandable => &mut workspace.expandable,
        Class::Materializable => &mut workspace.materializable,
    }
}

/// Top live entry of a class, discarding lazily-deleted ones.
fn peek(workspace: &mut Workspace, class: Class) -> Option<Entry> {
    loop {
        let top = *match class {
            Class::Bound => workspace.bound.peek(),
            Class::Expandable => workspace.expandable.peek(),
            Class::Materializable => workspace.materializable.peek(),
        }?;
        if workspace.is_stale(&top, class) {
            heap_mut(workspace, class).pop();
            continue;
        }
        return Some(top);
    }
}

/// Pop the top live entry of a CANDIDATE class.
///
/// A zero error is not a candidate. `certify_runtime_in` opens its scan with
/// `priority = 0.0` and selects only on a strict `score > priority`, so a cell that
/// already meets its demand is never refined however unrefined its siblings are.
/// Dropping that strictness silently over-refines and changes the certified frontier,
/// which is why the equivalence test caught it rather than a reviewer.
fn pop_live(workspace: &mut Workspace, class: Class) -> Option<Entry> {
    let entry = peek(workspace, class)?;
    if entry.error <= 0.0 && workspace.mutation != Mutation::ZeroErrorRefines {
        return None;
    }
    heap_mut(workspace, class).pop();
    Some(entry)
}

fn activate<const O: usize>(
    arena: &RuntimeArena,
    model: &mut impl CellwiseModel<O>,
    workspace: &mut Workspace,
    holon: usize,
) -> Result<(), IncrementalError> {
    use ciris_sim_core::holon::Decomposition;

    workspace.grow(arena.len());
    let record = *arena
        .holon(holon)
        .ok_or(IncrementalError::Holon(HolonError::InvalidParent))?;
    let error = model.cell_error(arena, holon);
    if !error.is_finite() || error < 0.0 {
        return Err(IncrementalError::CellErrorNotFinite);
    }

    workspace.active[holon] = true;
    workspace.stamp[holon] = workspace.stamp[holon].wrapping_add(1);
    workspace.error[holon] = error;
    let stamp = workspace.stamp[holon];
    let tie_key = if workspace.mutation == Mutation::HighestIdWinsTies {
        usize::MAX - holon
    } else {
        holon
    };
    workspace.bound.push(Entry {
        error,
        holon,
        tie_key,
        stamp,
    });

    if record.is_boundary() {
        if record.grain_units == 1 {
            workspace.at_floor += 1;
        }
        match record.decomposition {
            Decomposition::Expanded => workspace.expandable.push(Entry {
                error,
                holon,
                tie_key,
                stamp,
            }),
            Decomposition::Latent => workspace.materializable.push(Entry {
                error,
                holon,
                tie_key,
                stamp,
            }),
            Decomposition::Leaf => {}
        }
    }
    Ok(())
}

fn deactivate(workspace: &mut Workspace, holon: usize) {
    if !workspace.active[holon] {
        return;
    }
    workspace.active[holon] = false;
    // Bumping the stamp invalidates every heap entry this holon left behind, so the
    // lazy deletion in `peek` drops them without a scan.
    workspace.stamp[holon] = workspace.stamp[holon].wrapping_add(1);
}

/// A holon that stays active but can never be refined again (its materializer
/// declined). It keeps contributing to the bound and stops being a candidate.
///
/// This used to `retain` both candidate heaps, which is `O(n)` and rebuilds them. That
/// was invisible while a declined materialization was rare; pinning the scene at
/// observer acuity made declines routine, and two heap rebuilds per decline over a
/// 200,000-entry heap cost 24 SECONDS of a 25-second landscape throw. The lazy-deletion
/// machinery that already backs `peek` does the same job in `O(1)`: mark it, and let the
/// stale entry fall out when it reaches the top.
fn deactivate_as_terminal(workspace: &mut Workspace, holon: usize) {
    workspace.terminal[holon] = true;
}

fn refine<const O: usize>(
    arena: &RuntimeArena,
    model: &mut impl CellwiseModel<O>,
    workspace: &mut Workspace,
    holon: usize,
) -> Result<(), IncrementalError> {
    let record = *arena
        .holon(holon)
        .ok_or(IncrementalError::Holon(HolonError::InvalidParent))?;
    if record.is_boundary() && record.grain_units == 1 {
        workspace.at_floor = workspace.at_floor.saturating_sub(1);
    }
    deactivate(workspace, holon);
    if workspace.contiguous {
        let (first, count) = workspace.children[holon];
        for child in first..first + count {
            activate(arena, model, workspace, child as usize)?;
        }
        return Ok(());
    }
    // No range describes this arena's children, so fall back to the scan. Correct and
    // slow beats fast and missing a child.
    for child in 0..arena.len() {
        if arena.holons()[child].parent as usize == holon {
            activate(arena, model, workspace, child)?;
        }
    }
    Ok(())
}

/// Recompute every active cell error after the model bumped its epoch.
fn requench<const O: usize>(
    arena: &RuntimeArena,
    model: &mut impl CellwiseModel<O>,
    workspace: &mut Workspace,
) -> Result<(), IncrementalError> {
    collect_active(workspace);
    let active = workspace.active_list.clone();
    workspace.bound.clear();
    workspace.expandable.clear();
    workspace.materializable.clear();
    workspace.at_floor = 0;
    for holon in active {
        workspace.active[holon] = false;
        activate(arena, model, workspace, holon)?;
    }
    Ok(())
}

/// Drives the SHIPPED certifier from a [`CellwiseModel`], so the equivalence gate
/// compares two certifiers over one model rather than two models.
///
/// Two details make this a faithful mirror rather than a convenient one:
///
/// * `refinement_priority` takes `&self`, so cell errors are computed into a snapshot
///   during `evaluate` and read back there. That is the contract's second obligation —
///   priority ranks by cell error — made executable for the reference path.
/// * `settle` runs ONLY on a frontier whose bound already passes `macro_tolerance`,
///   which is what `FractureModel::evaluate` does with its cohesive solve. Settling on
///   every evaluation instead would call the model a different number of times on the
///   two paths, and any model whose field moves when it settles would then diverge for
///   a reason that has nothing to do with the certifier. That is not hypothetical: it
///   is what this test caught before the tolerance was threaded through.
pub struct PriorityAdapter<M> {
    pub model: M,
    macro_tolerance: f64,
    snapshot: Vec<f64>,
    last: Option<(Vec<f64>, f64)>,
}

impl<M> PriorityAdapter<M> {
    pub fn new(model: M, macro_tolerance: f64) -> Self {
        Self {
            model,
            macro_tolerance,
            snapshot: Vec::new(),
            last: None,
        }
    }
}

impl<const O: usize, M: CellwiseModel<O>> RuntimeBoundaryModel<O> for PriorityAdapter<M> {
    fn evaluate(&mut self, arena: &RuntimeArena, frontier: &RuntimeFrontier) -> Evaluation<O> {
        if self.snapshot.len() < arena.len() {
            self.snapshot.resize(arena.len(), 0.0);
        }
        let active: Vec<usize> = frontier.active_indices().collect();
        let mut bound = 0.0_f64;
        for holon in &active {
            let error = self.model.cell_error(arena, *holon);
            self.snapshot[*holon] = error;
            bound = bound.max(error);
        }
        if bound > self.macro_tolerance {
            let (observables, conservation_residual) = match &self.last {
                Some((observables, residual)) => {
                    let mut out = [0.0; O];
                    out.copy_from_slice(&observables[..O]);
                    (out, *residual)
                }
                None => ([0.0; O], 0.0),
            };
            return Evaluation {
                observables,
                macro_error_bound: bound,
                conservation_residual,
            };
        }

        let settled = self.model.settle(arena, &active, bound);
        self.last = Some((
            settled.observables.to_vec(),
            settled.conservation_residual,
        ));
        // The settle may have moved the field that produced the bound, so the bound is
        // re-read against the field the certificate will actually be issued under.
        let mut bound_after = 0.0_f64;
        for holon in &active {
            let error = self.model.cell_error(arena, *holon);
            self.snapshot[*holon] = error;
            bound_after = bound_after.max(error);
        }
        Evaluation {
            observables: settled.observables,
            macro_error_bound: bound_after,
            conservation_residual: settled.conservation_residual,
        }
    }

    fn refinement_priority(
        &self,
        _arena: &RuntimeArena,
        frontier: &RuntimeFrontier,
        holon: usize,
    ) -> f64 {
        if !frontier.is_active(holon) {
            return 0.0;
        }
        self.snapshot.get(holon).copied().unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ciris_sim_core::holon::{Channels, Decomposition};
    use ciris_sim_core::regplus::GrossState;
    use ciris_sim_core::runtime::{certify_runtime_adaptive, RuntimeHolonSpec};

    const FANOUT: usize = 4;

    /// Refines toward a target grain, with a spatially graded demand so that the
    /// descent is genuinely selective rather than uniform — a uniform demand would let
    /// a wrong tie-break pass unnoticed.
    struct Graded {
        target_grain: f64,
        /// Cells whose id is congruent to this are demanded harder, standing in for a
        /// corridor. Deliberately id-based so it is exactly reproducible.
        corridor: usize,
        epoch: u64,
        /// Bumps the epoch once, the round after this many settles, standing in for a
        /// damage surface that moves when a solve runs.
        move_after: Option<usize>,
        settles: usize,
        shifted: bool,
        /// How many times the certifier asked this model for a cell error. The work the
        /// certifier actually does, counted exactly.
        calls: usize,
    }

    impl Graded {
        fn new(target_grain: f64) -> Self {
            Self {
                target_grain,
                corridor: 3,
                epoch: 0,
                move_after: None,
                settles: 0,
                shifted: false,
                calls: 0,
            }
        }

        fn moving(target_grain: f64, after: usize) -> Self {
            let mut model = Self::new(target_grain);
            model.move_after = Some(after);
            model
        }

        /// Every cell demanded equally, so every round is a tie and only the tie-break
        /// decides which cell is refined. Without this the tie-break mutant would pass
        /// for the wrong reason.
        fn uniform(target_grain: f64) -> Self {
            let mut model = Self::new(target_grain);
            model.corridor = usize::MAX;
            model
        }

        fn demand(&self, holon: usize) -> f64 {
            let near = holon % 7 == self.corridor;
            if near ^ self.shifted {
                1.0
            } else {
                4.0
            }
        }
    }

    impl CellwiseModel<2> for Graded {
        fn cell_error(&mut self, arena: &RuntimeArena, holon: usize) -> f64 {
            self.calls += 1;
            let grain = arena.holons()[holon].grain_units as f64;
            (grain / (self.target_grain * self.demand(holon)) - 1.0).max(0.0)
        }

        fn settle(&mut self, _arena: &RuntimeArena, active: &[usize], bound: f64) -> Settled<2> {
            self.settles += 1;
            if let Some(after) = self.move_after {
                if self.settles == after && !self.shifted {
                    self.shifted = true;
                    self.epoch += 1;
                }
            }
            Settled {
                observables: [active.len() as f64, bound],
                conservation_residual: 0.0,
            }
        }

        fn epoch(&self) -> u64 {
            self.epoch
        }
    }

    /// Splits a latent holon into four, marking as NON-boundary any child already fine
    /// enough that no demand can ask more of it.
    ///
    /// That flag is not decoration. `GrainFloor` outranks `RefinementUnavailable`, so a
    /// single active boundary holon sitting at grain 1 halts adaptive materialization
    /// for the whole scene — which is precisely why the shipped
    /// `fracture::TipSpacingSelector` exists. A test materializer without the same rule
    /// would certify nothing and would be testing the wrong thing.
    struct Split {
        /// Declines every materialization once the arena reaches this size, which is
        /// how `RefinementUnavailable` is reached deliberately.
        cap: usize,
        /// Children at or below this grain are settled: flagged non-boundary.
        settled_grain: u32,
    }

    impl Split {
        fn new(cap: usize) -> Self {
            Self {
                cap,
                settled_grain: 1,
            }
        }

        /// Every child stays a boundary holon, including at the grain floor. This is
        /// the frontier shape a scene has BEFORE anyone writes a boundary selector, and
        /// it is the shape that makes the floor visible.
        fn boundary_to_the_floor(cap: usize) -> Self {
            Self {
                cap,
                settled_grain: 0,
            }
        }
    }

    impl RuntimeMaterializer for Split {
        fn materialize(
            &mut self,
            arena: &mut RuntimeArena,
            holon: usize,
        ) -> Result<bool, HolonError> {
            if arena.len() + FANOUT > self.cap {
                return Ok(false);
            }
            let record = *arena.holon(holon).ok_or(HolonError::InvalidParent)?;
            if record.decomposition != Decomposition::Latent || record.grain_units == 1 {
                return Ok(false);
            }
            let grain = (record.grain_units / 2).max(1);
            let decomposition = if grain == 1 {
                Decomposition::Leaf
            } else {
                Decomposition::Latent
            };
            let parts = FANOUT as u64;
            let base = record.gross.constituents / parts;
            let remainder = record.gross.constituents % parts;
            let specs: Vec<RuntimeHolonSpec<'_>> = (0..FANOUT)
                .map(|i| RuntimeHolonSpec {
                    parent: holon as u32,
                    depth: record.depth + 1,
                    grain_units: grain,
                    gross: GrossState::aggregate(
                        base + u64::from((i as u64) < remainder),
                        0,
                        [0, 0],
                    ),
                    whole: &[],
                    channels: record.channels,
                    boundary: grain > self.settled_grain,
                    decomposition,
                })
                .collect();
            arena.materialize(holon, &specs)?;
            Ok(true)
        }
    }

    fn root(root_grain: u32, constituents: u64) -> RuntimeArena {
        RuntimeArena::from_specs(
            &[RuntimeHolonSpec {
                parent: u32::MAX,
                depth: 0,
                grain_units: root_grain,
                gross: GrossState::aggregate(constituents, 0, [0, 0]),
                whole: &[],
                channels: Channels::MECHANICAL,
                boundary: true,
                decomposition: Decomposition::Latent,
            }],
            0,
        )
        .unwrap()
    }

    /// Run both certifiers over the same model and demand agreement. Returns the
    /// incremental certificate so callers can assert on the verdict too.
    /// Whether the incremental certificate agreed with the shipped one on every field
    /// that a certificate consists of.
    fn agreement(
        root_grain: u32,
        cap: usize,
        mutation: Mutation,
        make: impl Fn() -> Graded,
        split: impl Fn(usize) -> Split,
    ) -> (bool, IncrementalCertificate<2>) {
        let mut reference_arena = root(root_grain, 1_000_000);
        let mut reference = PriorityAdapter::new(make(), 0.0);
        let reference_certificate = certify_runtime_adaptive(
            &mut reference_arena,
            &mut reference,
            &mut split(cap),
            0.0,
            0.0,
        )
        .unwrap();

        let mut arena = root(root_grain, 1_000_000);
        let mut workspace = Workspace::with_mutation(mutation);
        let mut model = make();
        let budget = Budget {
            max_holons: cap,
            // A mutant can grow without bound (that is what dropping the GrainFloor
            // rank does), so the comparison run is capped. Hitting the cap IS a
            // disagreement, which is the right verdict for such a mutant.
            max_rounds: 200_000,
            ..Budget::new(0.0, 0.0)
        };
        let Ok(certificate) = certify_incremental(
            &mut arena,
            &mut model,
            &mut split(cap),
            &mut workspace,
            budget,
        ) else {
            return (
                false,
                IncrementalCertificate {
                    status: CertificationStatus::RefinementUnavailable,
                    observables: [f64::NAN; 2],
                    macro_error_bound: f64::NAN,
                    conservation_residual: f64::NAN,
                    rounds: 0,
                    materializations: 0,
                    active: Vec::new(),
                },
            );
        };

        let expected = &reference_certificate.certificate;
        let agrees = certificate.status == expected.status
            && (0..2).all(|k| {
                certificate.observables[k].to_bits() == expected.observables[k].to_bits()
            })
            && certificate.conservation_residual.to_bits()
                == expected.conservation_residual.to_bits()
            && certificate.macro_error_bound.to_bits() == expected.macro_error_bound.to_bits()
            && certificate.materializations == reference_certificate.materializations
            && certificate.active.len() == expected.frontier.active_count()
            && certificate
                .active
                .iter()
                .all(|holon| expected.frontier.is_active(*holon));
        (agrees, certificate)
    }

    fn equivalent(
        root_grain: u32,
        cap: usize,
        make: impl Fn() -> Graded,
    ) -> IncrementalCertificate<2> {
        let (agrees, certificate) =
            agreement(root_grain, cap, Mutation::None, make, Split::new);
        assert!(
            agrees,
            "incremental and shipped certificates disagreed at root_grain {root_grain}, \
             cap {cap}: {certificate:?}"
        );
        certificate
    }

    #[test]
    fn matches_the_shipped_certifier_bit_for_bit() {
        for root_grain in [2_u32, 4, 8, 16, 32, 64] {
            let certificate =
                equivalent(root_grain, usize::MAX, || Graded::new(1.0));
            assert!(certificate.passed());
        }
    }

    #[test]
    fn matches_the_shipped_certifier_when_the_model_field_moves() {
        // The epoch bump is the declared escape hatch for a model whose cell errors are
        // not a pure function of the arena. Both certifiers must survive it identically.
        for after in [1_usize, 2, 3] {
            equivalent(32, usize::MAX, || Graded::moving(1.0, after));
        }
    }

    #[test]
    fn reports_grain_floor_when_the_frontier_reaches_the_floor_unsatisfied() {
        // Target grain 0.25 is unreachable: grain_units is an integer and bottoms at 1,
        // so the demand can never be met and the boundary sits at the floor.
        let mut arena = root(8, 1_000_000);
        let mut workspace = Workspace::new();
        let mut model = Graded::new(0.25);
        let certificate = certify_incremental(
            &mut arena,
            &mut model,
            &mut Split::boundary_to_the_floor(usize::MAX),
            &mut workspace,
            Budget::new(0.0, 0.0),
        )
        .unwrap();
        assert_eq!(certificate.status, CertificationStatus::GrainFloor);

        let mut reference_arena = root(8, 1_000_000);
        let mut reference = PriorityAdapter::new(Graded::new(0.25), 0.0);
        let reference_certificate = certify_runtime_adaptive(
            &mut reference_arena,
            &mut reference,
            &mut Split::boundary_to_the_floor(usize::MAX),
            0.0,
            0.0,
        )
        .unwrap();
        assert_eq!(
            reference_certificate.certificate.status,
            CertificationStatus::GrainFloor
        );
    }

    #[test]
    fn reports_refinement_unavailable_when_the_materializer_declines() {
        // Cap the arena below what the demand needs, with the root still Latent and
        // above the grain floor: nothing is at the floor, so the honest verdict is
        // "children not resident", not "cannot go finer".
        let certificate = equivalent(64, 5, || Graded::new(1.0));
        assert_eq!(
            certificate.status,
            CertificationStatus::RefinementUnavailable
        );
    }

    #[test]
    fn replays_bit_identically() {
        let run = || {
            let mut arena = root(32, 1_000_000);
            let mut workspace = Workspace::new();
            let mut model = Graded::new(1.0);
            certify_incremental(
                &mut arena,
                &mut model,
                &mut Split::new(usize::MAX),
                &mut workspace,
                Budget::new(0.0, 0.0),
            )
            .unwrap()
        };
        assert_eq!(run(), run());
    }

    /// Every planted defect must be CAUGHT by the equivalence gate. A mutant that
    /// slips through would mean the gate is decorative — that is the whole point of
    /// running this, and `HighestIdWinsTies` in particular needs a scene with genuine
    /// ties or it would pass for the wrong reason.
    #[test]
    fn mutation() {
        // Ties are real here: `Uniform` gives every cell the same demand, so at each
        // round the whole active set is at one error and only the tie-break decides.
        let uniform = || Graded::uniform(1.0);
        let corridor = || Graded::new(1.0);
        // The field moves on the FIRST settle. A settle happens only on a frontier
        // whose bound already passes, and this descent settles once or twice, so a
        // later trigger never fires and would leave the epoch mutant inert — a mutant
        // that cannot act is not evidence that the gate catches it.
        let moving = || Graded::moving(1.0, 1);
        // A demand no grain can meet on the corridor cells, which are flagged
        // non-boundary once they hit the floor. The bound therefore stays above the
        // tolerance forever while every remaining CANDIDATE is already satisfied — the
        // only situation in which the zero-error guard changes anything at all.
        let unsatisfiable = || Graded::new(0.5);

        type Cases<'a> = [(
            Mutation,
            u32,
            usize,
            &'a dyn Fn() -> Graded,
            fn(usize) -> Split,
            &'a str,
        ); 5];
        let cases: Cases = [
            // Ties only CHANGE anything where the descent stops short, so the tie-break
            // case is capped: with every cell equally bad, which cells got refined
            // before the cap is reached is the only thing the order decides.
            (
                Mutation::HighestIdWinsTies,
                32,
                25,
                &uniform,
                Split::new,
                "tie-break",
            ),
            (
                Mutation::ZeroErrorRefines,
                16,
                usize::MAX,
                &unsatisfiable,
                Split::new,
                "zero-error guard",
            ),
            // The floor is only REACHABLE when children stay boundary holons all the
            // way down, and it only BITES when the demand at the floor is still unmet.
            (
                Mutation::IgnoreGrainFloorRank,
                16,
                usize::MAX,
                &unsatisfiable,
                Split::boundary_to_the_floor,
                "GrainFloor outranks RefinementUnavailable",
            ),
            (
                Mutation::SkipEpochRestart,
                32,
                usize::MAX,
                &moving,
                Split::new,
                "epoch restart",
            ),
            (
                Mutation::SettleAtDeadEnd,
                16,
                5,
                &corridor,
                Split::new,
                "dead-end readout",
            ),
        ];

        for (mutation, root_grain, cap, make, split, what) in cases {
            let (clean, _) = agreement(root_grain, cap, Mutation::None, make, split);
            assert!(clean, "{what}: the unmutated run must agree first");
            let (agrees, certificate) = agreement(root_grain, cap, mutation, make, split);
            assert!(
                !agrees,
                "{what} mutant ({mutation:?}) was NOT caught: {certificate:?}"
            );
        }
    }

    /// The whole reason this module exists, asserted on WORK DONE rather than wall
    /// clock — and on the right work, which took two attempts to find.
    ///
    /// The first fence timed two descents and demanded the slower be under 40x the
    /// faster. It fired at 40.5x on a machine running four other agents, passed five
    /// times once the load dropped, and would have flaked in CI forever.
    ///
    /// The second counted `rounds`. That was worse — it passed, but for no reason: the
    /// two certifiers make the SAME number of greedy steps (9, 37, 145, 581 at matched
    /// sizes), because the restart does not add rounds, it re-does the work inside each
    /// one. A fence that both the fixed and the broken implementation satisfy is not a
    /// fence.
    ///
    /// The quantity that separates them is how often the MODEL is asked for a cell
    /// error, and it separates them by four orders of magnitude. Measured, same model,
    /// same scenes:
    ///
    /// | holons | incremental calls/holon | shipped calls/holon |
    /// |---:|---:|---:|
    /// | 33 | 1.000 | 13.0 |
    /// | 145 | 1.000 | 180 |
    /// | 577 | 1.000 | 2,661 |
    /// | 2,321 | 1.000 | 42,323 |
    ///
    /// Exactly one call per holon, at every size: each holon's error is computed once,
    /// when it is activated. That is deterministic, load-immune, and it is the invariant
    /// a restored restart destroys immediately.
    #[test]
    fn asks_the_model_exactly_once_per_holon() {
        let run = |root_grain: u32| -> (usize, usize) {
            let mut arena = root(root_grain, 1_000_000);
            let mut workspace = Workspace::new();
            let mut model = Graded::new(1.0);
            let certificate = certify_incremental(
                &mut arena,
                &mut model,
                &mut Split::new(usize::MAX),
                &mut workspace,
                Budget::new(0.0, 0.0),
            )
            .unwrap();
            assert!(certificate.passed());
            (arena.len(), model.calls)
        };

        for root_grain in [16_u32, 32, 64, 128, 256] {
            let (holons, calls) = run(root_grain);
            assert_eq!(
                calls, holons,
                "at root grain {root_grain} the certifier asked the model {calls} times \
                 for {holons} holons; it must ask exactly once each, and anything more \
                 means work is being repeated"
            );
        }
    }

    #[test]
    fn rejects_a_non_finite_cell_error() {
        struct Broken;
        impl CellwiseModel<1> for Broken {
            fn cell_error(&mut self, _arena: &RuntimeArena, _holon: usize) -> f64 {
                f64::NAN
            }
            fn settle(&mut self, _: &RuntimeArena, _: &[usize], _: f64) -> Settled<1> {
                Settled {
                    observables: [0.0],
                    conservation_residual: 0.0,
                }
            }
        }
        let mut arena = root(8, 1_000_000);
        let mut workspace = Workspace::new();
        assert_eq!(
            certify_incremental(
                &mut arena,
                &mut Broken,
                &mut Split::new(usize::MAX),
                &mut workspace,
                Budget::new(0.0, 0.0),
            ),
            Err(IncrementalError::CellErrorNotFinite)
        );
    }

    #[test]
    fn round_budget_is_an_error_not_a_verdict() {
        let mut arena = root(64, 1_000_000);
        let mut workspace = Workspace::new();
        let mut model = Graded::new(1.0);
        let mut budget = Budget::new(0.0, 0.0);
        budget.max_rounds = 5;
        assert_eq!(
            certify_incremental(
                &mut arena,
                &mut model,
                &mut Split::new(usize::MAX),
                &mut workspace,
                budget,
            ),
            Err(IncrementalError::RoundBudgetExhausted)
        );
    }
}
