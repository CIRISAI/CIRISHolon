//! CIRISHolon sandbox: sand, in a box, at every zoom tier of one recursive holon.
//!
//! The browser owns pointer input and pixels. This crate owns every holon, every
//! materialization, every contact, and every certificate. Zoom is not a rendering
//! trick: each tier is a re-rooted [`ciris_sim_core::runtime::RuntimeArena`] with its
//! own declared grain constant, and the certificate says whether the coarse view is
//! honest at that tier or refuses.
//!
//! # The WebAssembly boundary
//!
//! JavaScript reads FLAT BUFFERS out of linear memory, not one call per field. The
//! shipped `holon-ball-game` exports a scalar getter per node coordinate, which is
//! clear at 288 nodes and is thousands of boundary crossings per frame at four
//! thousand. Everything per-frame here is packed into two `f32` arrays whose pointers
//! JavaScript reads once; everything per-event is a scalar; everything textual is a
//! single UTF-8 blob built once from the Rust tier table, so the words a reader sees
//! are the ones the engine holds.

pub mod chart;
pub mod gauge;
pub mod gravity;
pub mod incremental;
pub mod scene;
pub mod sim;
pub mod tier;

use std::sync::{Mutex, MutexGuard, OnceLock};

use scene::LawRefusal;
use sim::Session;
use ciris_sim_core::bridge::WeakFieldRefusal;
use tier::{Evaluator, Ledger, Refusal, TierId};

/// Floats per node in the render buffer: x, y, radius, anchored, speed.
///
/// The viewer declares this number too, because it reads the buffer as a flat
/// `Float32Array` and has to know how to cut it. Two declarations of one layout is a
/// half-landing waiting to happen — the buffer would still be well-formed, the reader
/// would still read a float, and every value would be the wrong field — so
/// `tests::the_viewer_cuts_the_frame_buffers_at_the_engine_s_stride` holds the pair.
pub const NODE_STRIDE: usize = 5;
/// Floats per relation in the render buffer: ax, ay, bx, by, damage. Mirrored in the
/// viewer and gated with [`NODE_STRIDE`].
pub const BOND_STRIDE: usize = 5;

struct World {
    session: Session,
    tier: TierId,
    grading: f64,
    /// Per-frame solver work budget, in node-substeps. Owned here so it survives the
    /// re-root a zoom or a throw performs.
    work_budget: usize,
    /// Which declared gravity scene is live, kept across the re-root a throw performs.
    gravity_scene: usize,
    nodes: Vec<f32>,
    bonds: Vec<f32>,
    text: Vec<u8>,
}

impl World {
    fn new() -> Self {
        let mut world = Self {
            session: Session::new(TierId::Sandbox),
            tier: TierId::Sandbox,
            grading: sim::GRADING,
            work_budget: sim::SUBSTEP_WORK_BUDGET,
            gravity_scene: 0,
            nodes: Vec::new(),
            bonds: Vec::new(),
            text: Vec::new(),
        };
        world.text = describe().into_bytes();
        world
    }

    fn publish(&mut self) {
        let nodes = self.session.nodes();
        self.nodes.clear();
        self.nodes.reserve(nodes.len() * NODE_STRIDE);
        for i in 0..nodes.len() {
            let speed = (nodes.velocity[i][0] * nodes.velocity[i][0]
                + nodes.velocity[i][1] * nodes.velocity[i][1])
                .sqrt();
            self.nodes.push(nodes.position[i][0] as f32);
            self.nodes.push(nodes.position[i][1] as f32);
            self.nodes.push(nodes.radius_m[i] as f32);
            self.nodes.push(if nodes.anchored[i] { 1.0 } else { 0.0 });
            self.nodes.push(speed as f32);
        }

        let relations = self.session.relations();
        self.bonds.clear();
        self.bonds.reserve(relations.len() * BOND_STRIDE);
        for (index, ends) in relations.ends.iter().enumerate() {
            let [a, b] = *ends;
            self.bonds.push(nodes.position[a][0] as f32);
            self.bonds.push(nodes.position[a][1] as f32);
            self.bonds.push(nodes.position[b][0] as f32);
            self.bonds.push(nodes.position[b][1] as f32);
            self.bonds.push(relations.bonds[index].damage() as f32);
        }
    }
}

fn world() -> MutexGuard<'static, World> {
    static WORLD: OnceLock<Mutex<World>> = OnceLock::new();
    WORLD
        .get_or_init(|| Mutex::new(World::new()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Escape the few characters a JSON string may not carry raw. The tier table's prose is
/// written in this repository and contains none of them today; the function exists so
/// that stays true of prose written later, rather than depending on it.
fn json_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if (control as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", control as u32));
            }
            other => out.push(other),
        }
    }
    out
}

/// A finite number as JSON, or `null`. JSON has no `NaN`, and the gauge tier's grain
/// and domain are deliberately not numbers — writing `NaN` would take the whole table
/// down with it, and writing a zero would be inventing a length the tier does not have.
fn json_number(value: f64) -> String {
    if value.is_finite() {
        format!("{value:e}")
    } else {
        "null".into()
    }
}

/// Everything textual the page needs, as one JSON object built from the Rust values so
/// the page cannot drift from the engine: the tier table, the declared gravity scenes,
/// and the weak-field refusal taxonomy WITH ITS UNLOCKS.
///
/// The unlocks come from `bridge::WeakFieldRefusal::unlock()` rather than being restated
/// here. A refusal whose unlock is retyped in the viewer is a refusal whose roadmap can
/// silently diverge from the engine's.
fn describe() -> String {
    let mut out = String::from("{\"tiers\":");
    out.push_str(&describe_tiers());
    out.push_str(",\"scenes\":");
    out.push_str(&describe_scenes());
    out.push_str(",\"weakField\":");
    out.push_str(&describe_weak_field());
    out.push('}');
    out
}

/// The declared gravity scenes, per tier index.
fn describe_scenes() -> String {
    let mut out = String::from("[");
    for (index, id) in TierId::ALL.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('[');
        for (n, scene) in gravity::scenes_for(*id).iter().enumerate() {
            if n > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "{{\"name\":\"{}\",\"plain\":\"{}\",\"stake\":{},\"view\":{}}}",
                json_escape(scene.name),
                json_escape(scene.plain),
                json_number(scene.tier_eps_max),
                json_number(scene.view_m),
            ));
        }
        out.push(']');
    }
    out.push(']');
    out
}

/// The weak-field refusal taxonomy, indexed by the code `ciris_weak_field_refusal`
/// returns, with each refusal's own unlock and whether it is a ceiling.
fn describe_weak_field() -> String {
    use ciris_sim_core::bridge::WeakFieldRefusal as R;
    let mut out = String::from("[{\"name\":\"\",\"unlock\":\"\",\"ceiling\":false}");
    for refusal in [
        R::ExceedsWeakField,
        R::ExpansionScale,
        R::UnsupportedPotentialFamily,
        R::RequiresSpacelikeSignal,
        R::Undeclarable,
    ] {
        out.push_str(&format!(
            ",{{\"name\":\"{:?}\",\"unlock\":\"{}\",\"ceiling\":{}}}",
            refusal,
            json_escape(refusal.unlock()),
            refusal.is_ceiling(),
        ));
    }
    out.push(']');
    out
}

/// The tier table as JSON, built from the Rust values so the page cannot drift from the
/// engine. Hand-rolled rather than pulled in as a dependency: this is one array of flat
/// records emitted once at start-up, and a serialization crate in a browser module that
/// is otherwise `ciris-sim-core` and nothing else is not worth the bytes.
fn describe_tiers() -> String {
    let mut out = String::from("[");
    for (index, tier) in tier::tiers().into_iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        let (evaluator, refusal) = match tier.evaluator {
            Evaluator::GaugePlaquette => ("exact plaquette", ""),
            Evaluator::GranularContact => ("granular contact", ""),
            Evaluator::Cohesive => ("cohesive relations", ""),
            // Not a refusal any more, and the text says what replaced it rather than
            // going quiet: weight pulls here because a certificate permits it.
            Evaluator::GeodesicChart { .. } => ("geodesic, weak-field chart", ""),
            Evaluator::Unavailable(Refusal::NoValidatedEvaluator) => (
                "none",
                "No validated evaluator exists at this tier in this repository. The \
                 crystal tier is specification-only: there are no force constants in \
                 the tree, and the density-functional calculation that would supply \
                 them is still running. The cell geometry is published measurement, \
                 which is data; running dynamics on it would be a number with no \
                 ancestor.",
            ),
            Evaluator::Unavailable(Refusal::NoGravityChart) => (
                "none",
                "This engine has no certified gravity. Its relativity chart is flat by \
                 design, and its own integration frame says every scene with weight is \
                 outside that certificate until the curved tier lands. So this tier has \
                 an exact ledger and no dynamics, and the honest thing is to show you \
                 the first and refuse the second.",
            ),
        };
        // The gauge tier has no length, so "this tier counted in atoms" is not a
        // question about it — the overflow factor comes back non-finite and the honest
        // answer is that the question does not apply, not a token spelling NaN.
        let atoms = match tier.ledger_in_atoms() {
            Ledger::Fits { constituents, .. } => format!("{constituents}"),
            Ledger::Overflows { factor, .. } | Ledger::OverflowsGrainUnits { factor }
                if factor.is_finite() =>
            {
                format!("over:{factor:e}")
            }
            _ => "n/a".into(),
        };
        out.push_str(&format!(
            "{{\"name\":\"{}\",\"plain\":\"{}\",\"g0\":{},\"domain\":{},\
             \"root\":{},\"constituents\":{},\"terminal\":\"{}\",\"evaluator\":\"{}\",\
             \"refusal\":\"{}\",\"unlock\":\"{}\",\"required\":{},\"lch\":{},\"atoms\":\"{}\"}}",
            json_escape(tier.name),
            json_escape(tier.plain),
            json_number(tier.g0_m),
            json_number(tier.domain_m),
            tier.root_grain_units,
            tier.constituents,
            json_escape(tier.terminal),
            evaluator,
            json_escape(refusal),
            json_escape(match tier.evaluator {
                Evaluator::Unavailable(refusal) => refusal.unlock(),
                _ => "",
            }),
            json_number(tier.required_spacing_m().unwrap_or(f64::NAN)),
            json_number(tier.characteristic_length_m().unwrap_or(f64::NAN)),
            atoms,
        ));
    }
    out.push(']');
    out
}

/// Pointer to the tier-table JSON in linear memory, and its length in bytes.
#[no_mangle]
pub extern "C" fn ciris_text_ptr() -> *const u8 {
    world().text.as_ptr()
}

#[no_mangle]
pub extern "C" fn ciris_text_len() -> u32 {
    world().text.len() as u32
}

/// Move to a tier. Re-roots the scene: a new arena with that tier's own grain constant.
#[no_mangle]
pub extern "C" fn ciris_set_tier(index: u32) {
    let Some(id) = TierId::from_index(index) else {
        return;
    };
    let mut world = world();
    let grading = world.grading;
    let budget = world.work_budget;
    world.tier = id;
    // A zoom is a re-root: the new tier picks its own first scene rather than inheriting
    // an index that meant something else one tier away.
    world.gravity_scene = 0;
    world.session = Session::with_grading(id, grading);
    world.session.set_work_budget(budget);
    world.publish();
}

#[no_mangle]
pub extern "C" fn ciris_tier() -> u32 {
    world().tier.index()
}

/// How fast the resolution demand relaxes with distance from the throw. Smaller demands
/// a finer frontier over a wider area, which is the control that makes the certified
/// frontier visible as a thing with a cost.
#[no_mangle]
pub extern "C" fn ciris_set_grading(grading: f64) {
    let mut world = world();
    world.grading = grading.clamp(0.05, 8.0);
}

#[no_mangle]
pub extern "C" fn ciris_grading() -> f64 {
    world().grading
}

/// Throw at `(aim_x, aim_y)` in domain fractions, at `speed` in 0..1.
#[no_mangle]
pub extern "C" fn ciris_throw(aim_x: f64, aim_y: f64, speed: f64) {
    let mut world = world();
    let id = world.tier;
    let grading = world.grading;
    // A throw re-roots the tier, so each one is certified from the root rather than
    // accumulated on top of the last throw's frontier. Replay is then a function of the
    // aim alone, which is what makes the same throw twice the same throw.
    let budget = world.work_budget;
    // The vacuum tier's flux is the SCENE, not a frontier, so it has to survive a throw
    // that re-roots everything else. Without this every raise started from zero flux and
    // the tier could never reach its own ceiling — the one refusal on the ladder that is
    // about the state space ending rather than about resolution simply never fired.
    let gauge = *world.session.gauge();
    let scene = world.gravity_scene;
    world.session = Session::with_grading(id, grading);
    world.session.set_work_budget(budget);
    world.session.set_gravity_scene(scene);
    *world.session.gauge_mut() = gauge;
    world.session.throw(aim_x, aim_y, speed);
    world.publish();
}

#[no_mangle]
pub extern "C" fn ciris_step(elapsed_seconds: f64) {
    let mut world = world();
    world.session.step(elapsed_seconds);
    world.publish();
}

#[no_mangle]
pub extern "C" fn ciris_node_ptr() -> *const f32 {
    world().nodes.as_ptr()
}

#[no_mangle]
pub extern "C" fn ciris_node_count() -> u32 {
    (world().nodes.len() / NODE_STRIDE) as u32
}

#[no_mangle]
pub extern "C" fn ciris_bond_ptr() -> *const f32 {
    world().bonds.as_ptr()
}

#[no_mangle]
pub extern "C" fn ciris_bond_count() -> u32 {
    (world().bonds.len() / BOND_STRIDE) as u32
}

/// Verdict code: 0 idle, 1 certified, 2 grain floor, 3 refinement unavailable,
/// 4 no evaluator, 5 no gravity chart, 6 budget exhausted, 7 flux ceiling.
#[no_mangle]
pub extern "C" fn ciris_verdict() -> u32 {
    world().session.verdict().code()
}

#[no_mangle]
pub extern "C" fn ciris_holons() -> u32 {
    world().session.arena().len() as u32
}

#[no_mangle]
pub extern "C" fn ciris_materializations() -> u32 {
    world().session.materializations() as u32
}

#[no_mangle]
pub extern "C" fn ciris_rounds() -> u32 {
    world().session.rounds() as u32
}

/// Set the solver's per-frame work budget, in node-substeps.
///
/// The engine cannot time itself on `wasm32-unknown-unknown`, so the host — which has a
/// clock — measures the frame and moves this toward its target. Nothing certified
/// depends on it: it changes how much SIMULATED time one frame buys, and the honest
/// consequence of that is reported by [`ciris_slow_motion`].
#[no_mangle]
pub extern "C" fn ciris_set_work_budget(budget: u32) {
    let mut world = world();
    world.work_budget = budget as usize;
    world.session.set_work_budget(budget as usize);
}

#[no_mangle]
pub extern "C" fn ciris_work_budget() -> u32 {
    world().work_budget as u32
}

/// Cells being integrated this frame. Resident and drawn is not the same as awake: a
/// scene at rest is at rest, and the acuity-pinned frontier is affordable precisely
/// because almost all of it is not moving.
#[no_mangle]
pub extern "C" fn ciris_awake_count() -> u32 {
    world().session.awake_count() as u32
}

/// Changes whenever a cell falls asleep. A host caching a rendering of the still cells
/// rebuilds it when this moves.
#[no_mangle]
pub extern "C" fn ciris_sleep_generation() -> f64 {
    world().session.sleep_generation() as f64
}

/// The observer's claim at the current tier, in metres: no cell in view is coarser.
#[no_mangle]
pub extern "C" fn ciris_acuity() -> f64 {
    world().session.tier.acuity_m()
}

#[no_mangle]
pub extern "C" fn ciris_substeps() -> u32 {
    world().session.substeps() as u32
}

/// Total contact impulse magnitude: how much pushing the throw did.
#[no_mangle]
pub extern "C" fn ciris_impulse() -> f64 {
    world().session.impulse_n_s()
}

/// Momentum the contact actually transferred — the conserved quantity, and the one the
/// certificate carries. It equals the projectile's own change in momentum, gated.
#[no_mangle]
pub extern "C" fn ciris_net_impulse() -> f64 {
    world().session.net_impulse_n_s()
}

#[no_mangle]
pub extern "C" fn ciris_peak_contact() -> f64 {
    world().session.peak_contact_n()
}

#[no_mangle]
pub extern "C" fn ciris_disturbance() -> f64 {
    world().session.disturbance_m()
}

#[no_mangle]
pub extern "C" fn ciris_cracked() -> u32 {
    world().session.relations().cracked() as u32
}

#[no_mangle]
pub extern "C" fn ciris_slow_motion() -> f64 {
    world().session.slow_motion()
}

#[no_mangle]
pub extern "C" fn ciris_softening() -> f64 {
    world().session.softening_factor()
}

#[no_mangle]
pub extern "C" fn ciris_dt() -> f64 {
    world().session.dt_s()
}

#[no_mangle]
pub extern "C" fn ciris_domain() -> f64 {
    world().session.tier.domain_m
}

#[no_mangle]
pub extern "C" fn ciris_projectile_x() -> f64 {
    world().session.projectile().position[0]
}

#[no_mangle]
pub extern "C" fn ciris_projectile_y() -> f64 {
    world().session.projectile().position[1]
}

#[no_mangle]
pub extern "C" fn ciris_projectile_r() -> f64 {
    world().session.projectile().radius_m
}

#[no_mangle]
pub extern "C" fn ciris_projectile_live() -> u32 {
    u32::from(world().session.projectile().live)
}

/// 0 none, 1 the homogenizer refused a cohesive law at this frontier's spacing (so the
/// relations are contact-only), 2 the tier declares no material chart.
#[no_mangle]
pub extern "C" fn ciris_law_refusal() -> u32 {
    match world().session.law_refusal() {
        None => 0,
        Some(LawRefusal::UnderResolved) => 1,
        Some(LawRefusal::NoMaterial) => 2,
    }
}

/// Order-of-magnitude strain rate this throw imposes, per second.
///
/// Read it against [`ciris_strain_gap_low`]/[`ciris_strain_gap_high`]: a value inside
/// that band is certified by INTERPOLATION between two families of experiment that do
/// not meet, and the page says so where the number is shown.
#[no_mangle]
pub extern "C" fn ciris_strain_rate() -> f64 {
    world().session.strain_rate_per_s()
}

#[no_mangle]
pub extern "C" fn ciris_strain_gap_low() -> f64 {
    sim::STRAIN_RATE_GAP.0
}

#[no_mangle]
pub extern "C" fn ciris_strain_gap_high() -> f64 {
    sim::STRAIN_RATE_GAP.1
}

/// The re-root relation between the current tier and the one above it, as a code:
/// 0 none, 1 exactly one parent terminal holon, 2 a whole multiple of them,
/// 3 contained within one.
#[no_mangle]
pub extern "C" fn ciris_reroot_kind() -> u32 {
    let world = world();
    let Some(parent) = TierId::from_index(world.tier.index() + 1) else {
        return 0;
    };
    match tier::reroot(world.tier, parent) {
        None => 0,
        Some(tier::Reroot::OneTerminalHolon { .. }) => 1,
        Some(tier::Reroot::WholeMultiple { .. }) => 2,
        Some(tier::Reroot::Contained { .. }) => 3,
    }
}

/// The re-root's number: child holons per parent terminal holon (kind 1), parent
/// terminal holons spanned (kind 2), or the fraction of one occupied (kind 3).
#[no_mangle]
pub extern "C" fn ciris_reroot_ratio() -> f64 {
    let world = world();
    let Some(parent) = TierId::from_index(world.tier.index() + 1) else {
        return 0.0;
    };
    match tier::reroot(world.tier, parent) {
        None => 0.0,
        Some(tier::Reroot::OneTerminalHolon { child_per_parent }) => child_per_parent,
        Some(tier::Reroot::WholeMultiple { parents }) => parents,
        Some(tier::Reroot::Contained { fraction }) => fraction,
    }
}

/// Select a declared gravity scene at the current tier. Out of range selects the first.
///
/// Each gravity tier carries two, because one number cannot say what the certificate
/// says: one that certifies and one that says something else — a licensed flat chart, or
/// a typed refusal.
#[no_mangle]
pub extern "C" fn ciris_set_gravity_scene(index: u32) {
    let mut world = world();
    world.gravity_scene = index as usize;
    world.session.set_gravity_scene(index as usize);
    // The screen is re-run for the new scene, so the panel never shows the last one's.
    world.session.settle();
    world.publish();
}

#[no_mangle]
pub extern "C" fn ciris_gravity_scene() -> u32 {
    world().session.gravity_scene_index() as u32
}

/// How many declared scenes this tier has. Zero for a tier with no gravity chart.
#[no_mangle]
pub extern "C" fn ciris_gravity_scene_count() -> u32 {
    gravity::scenes_for(world().tier).len() as u32
}

/// `max(|Phi|/c^2, (v/c)^2, (H L/c)^2)` over the declared envelope — what the screen
/// actually measured. Zero when the tier has no chart.
#[no_mangle]
pub extern "C" fn ciris_weak_field_epsilon() -> f64 {
    world()
        .session
        .weak_field()
        .map(|certificate| certificate.epsilon)
        .filter(|epsilon| epsilon.is_finite())
        .unwrap_or(0.0)
}

/// `K * eps^2` — the certified fractional remainder per dynamical time.
#[no_mangle]
pub extern "C" fn ciris_weak_field_remainder() -> f64 {
    world()
        .session
        .weak_field()
        .map(|certificate| certificate.remainder_bound)
        .filter(|remainder| remainder.is_finite())
        .unwrap_or(0.0)
}

/// The stake this scene was screened against.
#[no_mangle]
pub extern "C" fn ciris_weak_field_stake() -> f64 {
    world()
        .session
        .gravity_scene()
        .map(|scene| scene.tier_eps_max)
        .unwrap_or(0.0)
}

/// The cosmic background contribution alone, `(H L/c)^2`.
#[no_mangle]
pub extern "C" fn ciris_weak_field_background() -> f64 {
    world()
        .session
        .weak_field()
        .map(|certificate| certificate.epsilon_bg)
        .filter(|value| value.is_finite())
        .unwrap_or(0.0)
}

/// Typed weak-field refusal: 0 none, 1 exceeds weak field, 2 expansion scale,
/// 3 unsupported potential family, 4 requires spacelike signal (the one CEILING),
/// 5 undeclarable.
#[no_mangle]
pub extern "C" fn ciris_weak_field_refusal() -> u32 {
    match world().session.weak_field_refusal() {
        None => 0,
        Some(WeakFieldRefusal::ExceedsWeakField) => 1,
        Some(WeakFieldRefusal::ExpansionScale) => 2,
        Some(WeakFieldRefusal::UnsupportedPotentialFamily) => 3,
        Some(WeakFieldRefusal::RequiresSpacelikeSignal) => 4,
        Some(WeakFieldRefusal::Undeclarable) => 5,
    }
}

/// Is this refusal a CEILING — invariant under every re-root within the chart family —
/// rather than a floor a different chart would lift?
#[no_mangle]
pub extern "C" fn ciris_weak_field_is_ceiling() -> u32 {
    u32::from(
        world()
            .session
            .weak_field_refusal()
            .is_some_and(WeakFieldRefusal::is_ceiling),
    )
}

/// How wide the live gravity scene is to LOOK, in metres. Not the tier's domain: the
/// ledger's extent and the claim's extent are different numbers at these tiers.
#[no_mangle]
pub extern "C" fn ciris_gravity_view_m() -> f64 {
    world()
        .session
        .gravity_scene()
        .map(|scene| scene.view_m)
        .unwrap_or(0.0)
}

/// Points on the body's trail, in scene metres about the scene's own centre.
#[no_mangle]
pub extern "C" fn ciris_trail_count() -> u32 {
    world().session.trail().len() as u32
}

#[no_mangle]
pub extern "C" fn ciris_trail_ptr() -> *const f64 {
    let world = world();
    world.session.trail().as_ptr() as *const f64
}

/// Has a ballistic body come back to the height it was thrown from?
#[no_mangle]
pub extern "C" fn ciris_landed() -> u32 {
    u32::from(world().session.landed())
}

/// The vacuum tier's common loop flux: `-1`, `0` or `+1`.
#[no_mangle]
pub extern "C" fn ciris_gauge_flux() -> i32 {
    world().session.gauge().loop_flux() as i32
}

/// Ground-state probability of closed-flux sector `index` (0, 1, 2 for flux -1, 0, +1).
#[no_mangle]
pub extern "C" fn ciris_gauge_vacuum(index: u32) -> f64 {
    world()
        .session
        .gauge()
        .vacuum
        .get(index as usize)
        .copied()
        .unwrap_or(0.0)
}

/// `beta` in `-log rho_link = a + beta E²` — the one-link modular spectrum's electric
/// curvature, read out of the core's own eigensolve.
#[no_mangle]
pub extern "C" fn ciris_gauge_modular_beta() -> f64 {
    world().session.gauge().modular_beta
}

/// Departure of the vacuum from charge-conjugation symmetry. Measured, not assumed.
#[no_mangle]
pub extern "C" fn ciris_gauge_cc_residual() -> f64 {
    world().session.gauge().charge_conjugation_residual
}

/// Is the plaquette Gauss-closed? The core's machine predicate, so the demo reads the
/// law rather than its own bookkeeping.
#[no_mangle]
pub extern "C" fn ciris_gauge_closed() -> u32 {
    u32::from(world().session.gauge().closed())
}

/// Apply link charge conjugation, flux → −flux.
///
/// This is a DICTIONARY ENTRY, not a carrier identification: `Core/RouteGauge.lean`
/// killed the route → gauge carrier map by machine, and what survives is that this one
/// finite symmetry acts on the route Hamiltonian as time reversal. The page says exactly
/// that and no more.
#[no_mangle]
pub extern "C" fn ciris_gauge_conjugate() {
    world().session.gauge_mut().charge_conjugate();
}

/// Lower the loop by one unit of flux. The floor refuses exactly as the ceiling does.
#[no_mangle]
pub extern "C" fn ciris_gauge_lower() {
    world().session.gauge_mut().lower();
}

/// How many times over the REG+ ledger's `u64` this tier's domain would be if it were
/// counted in ATOMS instead of its own grain. Zero means it fits.
///
/// This is the zoom-out refusal, computed rather than asserted: the sandbox holds
/// 6.6e8 grains, which is 3.4e27 atoms, which is 1.9e8 times more than the ledger can
/// represent — and `GrossState::checked_combine` returns `None` rather than a wrong
/// number.
#[no_mangle]
pub extern "C" fn ciris_atom_overflow() -> f64 {
    match world().session.tier.ledger_in_atoms() {
        Ledger::Fits { .. } => 0.0,
        Ledger::Overflows { factor, .. } | Ledger::OverflowsGrainUnits { factor } => factor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim::Verdict;

    /// The viewer ships on the public site as the Sandbox tab (pages.yml copies
    /// `viewer/` verbatim), so its page must carry the way back: a reader who
    /// arrives from the tab bar and finds no link out is stranded in an exhibit.
    #[test]
    fn the_viewer_links_back_to_the_site() {
        let html = include_str!("../viewer/index.html");
        assert!(
            html.contains("href=\"../index.html\""),
            "viewer/index.html lost its back-link to the site root"
        );
    }

    /// The viewer cuts the frame buffers at the stride the engine packs them at.
    ///
    /// Both sides declare the number independently — the engine as a `usize`, the
    /// viewer as a `const` it uses to index a `Float32Array` — and until now nothing
    /// held them together. That is the one cross-boundary constant where disagreement
    /// is SILENT: no length check fails, no float is out of range, the page simply
    /// draws radius as a position. `the_frame_buffers_are_packed_at_their_declared_
    /// stride` gates the Rust side against itself; this gates the browser's copy
    /// against the Rust side, which is the half that a stride change actually moves.
    ///
    /// The `viewer/` directory ships verbatim to the public site (pages.yml), so the
    /// file this reads is the file that runs.
    ///
    /// Adjacent and deliberately not covered here: the viewer also mirrors the verdict
    /// codes and the re-root kinds. Those fail LOUDLY — an unknown code renders as a
    /// missing label — and widening this gate to cover them is a separate change.
    #[test]
    fn the_viewer_cuts_the_frame_buffers_at_the_engine_s_stride() {
        let js = include_str!("../viewer/app.js");
        for (name, stride) in [("NODE_STRIDE", NODE_STRIDE), ("BOND_STRIDE", BOND_STRIDE)] {
            let declaration = format!("const {name} = {stride};");
            assert!(
                js.contains(&declaration),
                "viewer/app.js does not declare `{declaration}`; the engine packs \
                 {name} = {stride} and the viewer would cut the buffer somewhere else"
            );
        }
    }

    /// The exported surface reads and writes ONE world, so tests that drive it are a
    /// sequence of sessions on a shared machine, not independent functions: under the
    /// default parallel runner, one test's `ciris_set_tier` lands between another's
    /// set and its assert. The gauge tier made this bite — its flux now survives a
    /// throw by design, which is persistent cross-call state. Each world-driving test
    /// takes this lock for its whole body. A failing test poisons the mutex; the next
    /// test recovers the guard rather than dying of `PoisonError`, so the first real
    /// failure stays the only failure reported.
    fn world_test_lock() -> MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    #[test]
    fn the_tier_table_reaches_the_browser_as_the_engine_holds_it() {
        let text = describe();
        assert!(text.starts_with('{'), "{text}");
        assert!(text.ends_with('}'));
        for tier in tier::tiers() {
            assert!(text.contains(tier.name), "the tier table lost {}", tier.name);
            assert!(
                text.contains(tier.terminal),
                "the tier table lost what {}'s terminal holon is",
                tier.name
            );
        }
        // The refusals have to survive the trip, or the demo's most honest feature is
        // the one that silently does not ship.
        assert!(text.contains("No validated evaluator exists"));
        // The gravity refusal text is GONE from the tier table, because the tiers that
        // carried it no longer refuse wholesale. What must survive instead is the
        // sentence that replaced it.
        assert!(!text.contains("no certified way to make weight pull"));
        assert!(
            text.contains("weight finally") && text.contains("geodesic"),
            "the planet tier must say what replaced its refusal"
        );
        // A refusal that does not name its unlock is a shortfall rather than a
        // roadmap, so both unlocks are required to survive the trip too.
        assert!(text.contains("Awaits the T2 gate"));
        // "Awaits the curved-tier certificate" is GONE, and its absence is the point:
        // that unlock was discharged when the bridge landed. What must reach the browser
        // in its place is the weak-field taxonomy's own unlocks, quoted from the bridge
        // rather than retyped here.
        assert!(!text.contains("Awaits the curved-tier certificate"));
        assert!(text.contains("Awaits the FRW chart family"));
        assert!(text.contains("Awaits a strong-field chart family"));
        assert!(text.contains("v2 logarithmic-potential family"));
        assert!(
            text.contains("Nothing lifts this at any tier"),
            "the one CEILING must survive the boundary too — it is the only refusal on \
             the whole ladder that no re-root can lift"
        );
        // And every declared gravity scene must arrive with its own words.
        for id in [TierId::Planet, TierId::Galactic, TierId::Cosmic] {
            for scene in gravity::scenes_for(id) {
                assert!(text.contains(scene.name), "lost scene {}", scene.name);
            }
        }
        // JSON has no NaN. The gauge tier has no grain and no domain, and both of those
        // must arrive as `null` rather than as a token that takes the table down.
        assert!(!text.contains("NaN"), "the tier table emitted a bare NaN");
        assert!(text.contains("\"g0\":null"), "the gauge tier must have no length");
    }

    /// Every exported function takes the world lock for the duration of its own call
    /// and no longer, so two of them may appear in one expression. Writing
    /// `assert_eq!(world().nodes.len(), ciris_node_count() ...)` holds the first guard
    /// across the second lock and deadlocks the test binary — silently, as a hang.
    /// The buffers are therefore read into locals first, which is also what the
    /// JavaScript side does.
    #[test]
    fn the_frame_buffers_are_packed_at_their_declared_stride() {
        let _world = world_test_lock();
        ciris_set_tier(TierId::Sandbox.index());
        ciris_throw(0.5, 0.4, 0.6);
        let nodes = ciris_node_count() as usize;
        assert!(nodes > 0);
        assert_eq!(world().nodes.len(), nodes * NODE_STRIDE);
        let bonds = ciris_bond_count() as usize;
        assert_eq!(world().bonds.len(), bonds * BOND_STRIDE);
        assert_eq!(ciris_verdict(), Verdict::Certified.code());
        ciris_step(1.0 / 60.0);
        let stepped = ciris_node_count() as usize;
        assert_eq!(world().nodes.len(), stepped * NODE_STRIDE);
    }

    #[test]
    fn the_zoom_out_refusal_is_computed_and_reaches_the_browser() {
        let _world = world_test_lock();
        ciris_set_tier(TierId::Sandbox.index());
        let factor = ciris_atom_overflow();
        assert!(
            (1.0e8..1.0e9).contains(&factor),
            "the sandbox in atoms should be ~1.9e8x over the ledger, got {factor:e}"
        );
        ciris_set_tier(TierId::Grain.index());
        assert_eq!(
            ciris_atom_overflow(),
            0.0,
            "one grain of sand IS countable in atoms"
        );
    }

    /// The vacuum tier's flux persists across throws and reaches its ceiling, and
    /// moving tiers resets it. Both halves matter: without persistence the ceiling
    /// never fires, and without the reset a tier would remember another tier's state.
    #[test]
    fn the_vacuum_tier_reaches_its_ceiling_and_resets_on_zoom() {
        let _world = world_test_lock();
        ciris_set_tier(TierId::Gauge.index());
        assert_eq!(ciris_gauge_flux(), 0);
        ciris_throw(0.5, 0.5, 0.6);
        assert_eq!(ciris_gauge_flux(), 1);
        assert_eq!(ciris_verdict(), Verdict::Certified.code());
        ciris_throw(0.5, 0.5, 0.6);
        assert_eq!(ciris_verdict(), Verdict::FluxCeiling.code());
        assert_eq!(ciris_gauge_flux(), 1, "a refused move must not move anything");
        assert_eq!(ciris_gauge_closed(), 1, "and the Gauss law still holds");

        // Charge conjugation is the way back down, and it is a dictionary entry rather
        // than a carrier identification (see `gauge`'s header).
        ciris_gauge_conjugate();
        assert_eq!(ciris_gauge_flux(), -1);
        ciris_throw(0.5, 0.5, 0.6);
        assert_eq!(ciris_gauge_flux(), 0);
        assert_eq!(ciris_verdict(), Verdict::Certified.code());

        // Zooming away and back is a re-root, and a re-root starts clean.
        ciris_set_tier(TierId::Sandbox.index());
        ciris_set_tier(TierId::Gauge.index());
        assert_eq!(ciris_gauge_flux(), 0);
    }

    /// A certificate never survives a zoom, and a throw never crosses one.
    ///
    /// `CIRISOntology/Core/GrainFloor.lean` proves `cert_does_not_transport_across_reroot`:
    /// a claim served at one floor and refused at another means a certificate earned on
    /// one side of a re-root states NOTHING on the other. The engine has to honour that
    /// structurally rather than by convention, so: zooming builds a new arena and the
    /// previous verdict and observables are discarded, and a throw is always evaluated
    /// inside one tier because it re-roots at the current tier before certifying. There
    /// is no code path that certifies across a re-root, and this is what keeps it that
    /// way.
    #[test]
    fn a_certificate_does_not_survive_a_zoom() {
        let _world = world_test_lock();

        // Earn a real certificate at the sandbox.
        ciris_set_tier(TierId::Sandbox.index());
        ciris_throw(0.5, 0.4, 0.6);
        for _ in 0..30 {
            ciris_step(1.0 / 60.0);
        }
        assert_eq!(ciris_verdict(), Verdict::Certified.code());
        assert!(ciris_net_impulse() > 0.0, "nothing was transferred to certify about");

        // Zoom. The verdict and every observable must be gone, not inherited.
        ciris_set_tier(TierId::Grain.index());
        assert_eq!(
            ciris_verdict(),
            Verdict::Idle.code(),
            "a zoom carried the previous tier's verdict across a re-root"
        );
        assert_eq!(ciris_net_impulse(), 0.0, "a zoom carried an impulse across");
        assert_eq!(ciris_impulse(), 0.0);
        assert_eq!(ciris_disturbance(), 0.0);

        // And zooming to a tier that refuses states the refusal, rather than inheriting
        // the certificate that was valid one tier away.
        ciris_set_tier(TierId::Sandbox.index());
        ciris_throw(0.5, 0.4, 0.6);
        assert_eq!(ciris_verdict(), Verdict::Certified.code());
        // Zooming to a gravity tier shows THAT tier's screen, computed on arrival —
        // never the certificate that was valid one tier away.
        ciris_set_tier(TierId::Planet.index());
        assert_eq!(ciris_verdict(), Verdict::Certified.code());
        assert!(
            ciris_weak_field_epsilon() > 0.0,
            "the planet's verdict must come with its own measured epsilon"
        );
    }

    #[test]
    fn every_tier_can_be_selected_and_thrown_at() {
        let _world = world_test_lock();
        for id in TierId::ALL {
            ciris_set_tier(id.index());
            assert_eq!(ciris_tier(), id.index());
            ciris_throw(0.5, 0.4, 0.6);
            for _ in 0..10 {
                ciris_step(1.0 / 60.0);
            }
            assert!(ciris_holons() >= 1);
            assert!(ciris_impulse().is_finite());
        }
    }
}
