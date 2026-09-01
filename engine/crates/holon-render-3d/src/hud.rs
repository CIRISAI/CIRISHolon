//! The overlay: the ledger, both gates, the three clocks, the holon census and the
//! device measurement, plus the controls.
//!
//! Every number here is READ from the `Sim` and none is recomputed. That is the point of
//! the overlay rather than a detail of it: the canvas shell and this one are two views of
//! one core, so if a gate reads PASS in the browser and FAIL here, the difference is a
//! real difference in the run and not a difference in the arithmetic of two HUDs.
//!
//! The panel set mirrors `docs/atoms/index.html` section for section — GATE 1 · ENERGY,
//! GATE 2 · MOMENTUM, THE THREE CLOCKS, THE CURVE, HOLON CENSUS, THIS DEVICE — because
//! somebody who has read one shell should not have to learn the other.
//!
//! One gate per conservation law, never combined. A single "is the simulation OK" light
//! can be green while energy is right and momentum is five times wrong, so energy and
//! momentum are shown separately, against separately derived bounds, and each shows its
//! measured-over-bound ratio so a passing gate still displays its margin.

use bevy::prelude::*;
use holon_render::clock::AU_TO_FS;
use holon_render::sim::{Boundary, DEFAULT_SCENE_ATOMS};

use crate::scene::{AMBER, MUTED, PAPER, RUST};
use crate::world::{AtomWorld, Calibration};

/// Which readout a text entity carries. One system fills them all.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Status,
    EnergyVerdict,
    EnergyBody,
    MomentumVerdict,
    MomentumBody,
    RungVerdict,
    ClockBody,
    CurveBody,
    CensusBody,
    DeviceBody,
    LabelAtoms,
    LabelSpeed,
    LabelWalls,
    LabelThermostat,
    LabelCensus,
    LabelDtGrowth,
    LabelPreset,
}

/// What a control button does.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Reset,
    NextPreset,
    FewerAtoms,
    MoreAtoms,
    Slower,
    Faster,
    ToggleWalls,
    ToggleThermostat,
    ToggleCensus,
    ToggleDtGrowth,
    ToggleHud,
}

/// Whether the panels are shown. The scene is the point; on a phone the overlay can be
/// most of the screen, so it folds away to a single button.
#[derive(Resource)]
pub struct HudVisible(pub bool);

/// Marker on the two side columns, so [`Action::ToggleHud`] can hide them.
#[derive(Component)]
struct HudPanel;

/// Multiplicative step per press of the sim-speed buttons. A ratio rather than an
/// increment because the useful range spans three decades.
const SPEED_STEP: f64 = 1.5;

const PANEL_BG: Color = Color::srgba(0.055, 0.086, 0.078, 0.86);
const HEADER_SIZE: f32 = 10.0;
const BODY_SIZE: f32 = 11.5;
const VERDICT_SIZE: f32 = 15.0;

pub fn plugin(app: &mut App) {
    app.insert_resource(HudVisible(true))
        .add_systems(Startup, setup_hud)
        .add_systems(Update, (handle_actions, update_hud, button_visuals));
}

// ------------------------------------------------------------------ construction

fn setup_hud(mut commands: Commands) {
    // Transparent full-screen root, laid out as two columns with the scene between them.
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            GlobalZIndex(10),
        ))
        .id();

    // ── left column: the two gates and the clocks ─────────────────────────
    let left = column(&mut commands, root);
    commands.entity(left).insert(HudPanel);
    let energy = panel(&mut commands, left, "GATE 1 / ENERGY");
    verdict(&mut commands, energy, Slot::EnergyVerdict);
    body(&mut commands, energy, Slot::EnergyBody);
    let momentum = panel(&mut commands, left, "GATE 2 / MOMENTUM");
    verdict(&mut commands, momentum, Slot::MomentumVerdict);
    body(&mut commands, momentum, Slot::MomentumBody);
    let clocks = panel(&mut commands, left, "THE THREE CLOCKS");
    verdict(&mut commands, clocks, Slot::RungVerdict);
    body(&mut commands, clocks, Slot::ClockBody);

    // ── right column: the curve, the census, the device ───────────────────
    let right = column(&mut commands, root);
    commands.entity(right).insert(HudPanel);
    let curve = panel(&mut commands, right, "THE CURVE / U(R)");
    body(&mut commands, curve, Slot::CurveBody);
    let census = panel(&mut commands, right, "HOLON CENSUS");
    body(&mut commands, census, Slot::CensusBody);
    let device = panel(&mut commands, right, "THIS DEVICE (MEASURED)");
    body(&mut commands, device, Slot::DeviceBody);

    // ── the bottom bar: status line and controls ──────────────────────────
    let bar = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            GlobalZIndex(11),
        ))
        .id();
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: BODY_SIZE.into(),
            ..default()
        },
        TextColor(PAPER),
        Slot::Status,
        ChildOf(bar),
    ));
    let row = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(4.0),
                row_gap: Val::Px(4.0),
                ..default()
            },
            ChildOf(bar),
        ))
        .id();
    button(&mut commands, row, Action::Reset, "reset", None);
    button(
        &mut commands,
        row,
        Action::NextPreset,
        "preset: H₂",
        Some(Slot::LabelPreset),
    );
    button(&mut commands, row, Action::FewerAtoms, "-", None);
    button(
        &mut commands,
        row,
        Action::MoreAtoms,
        "+",
        Some(Slot::LabelAtoms),
    );
    button(&mut commands, row, Action::Slower, "slower", None);
    button(
        &mut commands,
        row,
        Action::Faster,
        "faster",
        Some(Slot::LabelSpeed),
    );
    button(
        &mut commands,
        row,
        Action::ToggleWalls,
        "walls",
        Some(Slot::LabelWalls),
    );
    button(
        &mut commands,
        row,
        Action::ToggleThermostat,
        "thermostat",
        Some(Slot::LabelThermostat),
    );
    button(
        &mut commands,
        row,
        Action::ToggleCensus,
        "composite",
        Some(Slot::LabelCensus),
    );
    button(
        &mut commands,
        row,
        Action::ToggleDtGrowth,
        "dt",
        Some(Slot::LabelDtGrowth),
    );
    button(&mut commands, row, Action::ToggleHud, "hud", None);
}

fn column(commands: &mut Commands, parent: Entity) -> Entity {
    commands
        .spawn((
            Node {
                width: Val::Px(268.0),
                max_width: Val::Percent(46.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
            ChildOf(parent),
        ))
        .id()
}

fn panel(commands: &mut Commands, parent: Entity, header: &str) -> Entity {
    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(3.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(PANEL_BG),
            BorderColor::all(Color::srgba(0.44, 0.65, 0.56, 0.28)),
            // Present so the focus system reports a pointer over the panel; `pick.rs`
            // reads it and declines to grab an atom through the overlay.
            Interaction::default(),
            ChildOf(parent),
        ))
        .id();
    commands.spawn((
        Text::new(header.to_string()),
        TextFont {
            font_size: HEADER_SIZE.into(),
            ..default()
        },
        TextColor(MUTED),
        ChildOf(panel),
    ));
    panel
}

fn verdict(commands: &mut Commands, parent: Entity, slot: Slot) {
    commands.spawn((
        Text::new("-"),
        TextFont {
            font_size: VERDICT_SIZE.into(),
            ..default()
        },
        TextColor(PAPER),
        slot,
        ChildOf(parent),
    ));
}

fn body(commands: &mut Commands, parent: Entity, slot: Slot) {
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: BODY_SIZE.into(),
            ..default()
        },
        TextColor(PAPER),
        slot,
        ChildOf(parent),
    ));
}

/// Per-interaction background colours, so a press reads on a touch screen where there is
/// no hover state to fall back on.
#[derive(Component, Clone, Copy)]
struct ButtonColors {
    normal: Color,
    hover: Color,
    pressed: Color,
}

fn button(
    commands: &mut Commands,
    parent: Entity,
    action: Action,
    label: &str,
    slot: Option<Slot>,
) {
    let colors = ButtonColors {
        normal: Color::srgba(0.137, 0.412, 0.341, 0.85),
        hover: Color::srgba(0.20, 0.52, 0.44, 0.92),
        pressed: Color::srgba(0.83, 0.60, 0.21, 0.95),
    };
    let b = commands
        .spawn((
            Button,
            Node {
                // Generously tall: this is a phone target, and 34 px is about the
                // smallest a fingertip hits reliably.
                padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
                min_height: Val::Px(34.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(colors.normal),
            colors,
            action,
            ChildOf(parent),
        ))
        .id();
    let text = commands
        .spawn((
            Text::new(label.to_string()),
            TextFont {
                font_size: BODY_SIZE.into(),
                ..default()
            },
            TextColor(PAPER),
            ChildOf(b),
        ))
        .id();
    if let Some(slot) = slot {
        commands.entity(text).insert(slot);
    }
}

fn button_visuals(
    mut query: Query<(&Interaction, &ButtonColors, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, colors, mut background) in &mut query {
        background.0 = match interaction {
            Interaction::Pressed => colors.pressed,
            Interaction::Hovered => colors.hover,
            Interaction::None => colors.normal,
        };
    }
}

// ------------------------------------------------------------------ the controls

fn handle_actions(
    actions: Query<(&Interaction, &Action), Changed<Interaction>>,
    mut world: ResMut<AtomWorld>,
    mut visible: ResMut<HudVisible>,
    mut panels: Query<&mut Visibility, With<HudPanel>>,
) {
    for (interaction, action) in &actions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            Action::Reset => {
                let n = world.sim.n;
                world.reset(n);
            }
            Action::NextPreset => {
                world.next_preset();
            }
            Action::FewerAtoms => {
                let n = world.sim.n.saturating_sub(1).max(2);
                world.reset(n);
            }
            Action::MoreAtoms => {
                let n = (world.sim.n + 1).min(DEFAULT_SCENE_ATOMS);
                world.reset(n);
            }
            Action::Slower => {
                world.sim.timescale.sim_speed_fs_per_wallsec /= SPEED_STEP;
            }
            Action::Faster => {
                world.sim.timescale.sim_speed_fs_per_wallsec *= SPEED_STEP;
            }
            Action::ToggleWalls => {
                world.sim.boundary = match world.sim.boundary {
                    Boundary::Walls => Boundary::Open,
                    Boundary::Open => Boundary::Walls,
                };
                // The ledger's origin is a property of the scene, and turning a wall off
                // changes the scene. Re-basing here is what keeps the drift reading a
                // measurement of the integrator rather than of the edit.
                world.sim.rebase();
            }
            Action::ToggleThermostat => {
                world.sim.thermostat_on = !world.sim.thermostat_on;
            }
            Action::ToggleCensus => {
                world.sim.holons.enabled = !world.sim.holons.enabled;
            }
            Action::ToggleDtGrowth => {
                // Rung (ii), and only by explicit toggle. Turning it OFF re-derives dt
                // from the envelope immediately rather than leaving the enlarged step in
                // place — the same sequence the browser ABI performs.
                let on = !world.sim.timescale.allow_dt_growth;
                world.sim.timescale.allow_dt_growth = on;
                if on {
                    world.sim.timescale.set_dt_multiplier(2.0);
                } else {
                    let e = world.sim.timescale.e_rel_max;
                    world.sim.timescale.e_rel_max = f64::NEG_INFINITY;
                    world.sim.timescale.k_env = 0.0;
                    let table = core::mem::replace(
                        world.sim.table_mut(),
                        holon_render::table::PotentialTable::empty(),
                    );
                    world.sim.timescale.refresh_envelope(&table, e);
                    *world.sim.table_mut() = table;
                }
            }
            Action::ToggleHud => {
                visible.0 = !visible.0;
                for mut v in &mut panels {
                    *v = if visible.0 {
                        Visibility::Inherited
                    } else {
                        Visibility::Hidden
                    };
                }
            }
        }
    }
}

// ------------------------------------------------------------------ the readouts

fn update_hud(world: Res<AtomWorld>, mut texts: Query<(&Slot, &mut Text, &mut TextColor)>) {
    let s = &world.sim;
    let energy_pass = s.energy_gate();
    let momentum_pass = s.momentum_gate();
    let drift_bound = s.drift_bound();
    let momentum_bound = s.momentum_bound();
    let (rung, rung_note) = world.rung_label();
    let (px, py, pz) = s.momentum();
    let p_mag = (px * px + py * py + pz * pz).sqrt();

    for (slot, mut text, mut color) in &mut texts {
        match slot {
            Slot::Status => {
                text.0 = if !world.table_ok() {
                    format!(
                        "THE CURVE DID NOT LOAD (status {}) - nothing is being integrated",
                        world.table_status
                    )
                } else {
                    format!(
                        "[{}]  t = {:.1} a.u. ({:.3} fs)  |  {} atoms  |  {} clusters  |  {} bonded pairs  |  \
                         drag an atom; the spring is on the ledger",
                        world.preset.short_name(),
                        s.time,
                        s.time * AU_TO_FS,
                        s.n,
                        s.cluster_count().0,
                        s.bonded_count(),
                    )
                };
                color.0 = if world.table_ok() { PAPER } else { RUST };
            }
            Slot::EnergyVerdict => {
                text.0 = format!(
                    "{}  E - W_ext is constant",
                    if energy_pass { "PASS" } else { "FAIL" }
                );
                color.0 = if energy_pass { GREEN_TEXT } else { RUST };
            }
            Slot::EnergyBody => {
                text.0 = format!(
                    "E_kin      {:+.6e}\n\
                     E_pair     {:+.6e}\n\
                     E_wall     {:+.6e}\n\
                     E_spring   {:+.6e}\n\
                     W_ext      {:+.6e}  (your hand)\n\
                     E - W_ext  {:+.6e}\n\
                     origin     {:+.6e}\n\
                     drift peak  {:.3e}\n\
                     bound       {:.3e}\n\
                     ratio       {:.4}",
                    s.e_kin,
                    s.e_pair,
                    s.e_wall,
                    s.e_spring,
                    s.w_ext,
                    s.ledger(),
                    s.l0,
                    s.drift_peak,
                    drift_bound,
                    ratio(s.drift_peak, drift_bound),
                );
            }
            Slot::MomentumVerdict => {
                text.0 = format!(
                    "{}  P - J_ext is constant",
                    if momentum_pass { "PASS" } else { "FAIL" }
                );
                color.0 = if momentum_pass { GREEN_TEXT } else { RUST };
            }
            Slot::MomentumBody => {
                text.0 = format!(
                    "|P|        {:.6e}\n\
                     Px         {:+.4e}\n\
                     Py         {:+.4e}\n\
                     Pz         {:+.4e}\n\
                     residual   {:.3e}\n\
                     roundoff   {:.3e}\n\
                     ratio      {:.4}\n\
                     A separate gate on purpose: energy\n\
                     can be right while momentum is wrong.",
                    p_mag,
                    px,
                    py,
                    pz,
                    s.momentum_residual_peak,
                    momentum_bound,
                    ratio(s.momentum_residual_peak, momentum_bound),
                );
            }
            Slot::RungVerdict => {
                text.0 = format!("{rung}\n{rung_note}");
                color.0 = match rung {
                    "EXACT" => GREEN_TEXT,
                    "REFUSED" => RUST,
                    _ => AMBER,
                };
            }
            Slot::ClockBody => {
                let t = &s.timescale;
                // No leading whitespace on any continuation line: a `\` line-continuation
                // in a Rust string literal eats the indentation that follows it, so the
                // sub-rows are marked with a rule character instead of indented.
                text.0 = format!(
                    "1 | physics dt  {:.5} a.u. (derived)\n\
                     |  reference    {:.5} = period/64\n\
                     |  w_e at R_e   {:.6}\n\
                     |  w_env        {:.6} (envelope)\n\
                     |  w_env * dt   {:.5} (<2 stable)\n\
                     2 | frame       {:.1} ms\n\
                     |  substeps     {}\n\
                     3 | sim-speed   {:.4} fs/s\n\
                     |  1 vibration  {:.2} s\n\
                     |  dilation     {:.4}",
                    t.dt,
                    t.dt_reference,
                    t.omega_e,
                    t.omega_env,
                    t.omega_dt(),
                    world.last_frame_seconds * 1000.0,
                    world.last_substeps,
                    t.sim_speed_fs_per_wallsec,
                    world.wall_seconds_per_vibration(),
                    t.dilation,
                );
            }
            Slot::CurveBody => {
                // The closest pair is the one worth quoting: it is the one about to do
                // something.
                let closest = s.pairs[..s.pair_count]
                    .iter()
                    .min_by(|a, b| a.r.total_cmp(&b.r));
                let pair = match closest {
                    Some(p) => {
                        let sym_i = s.atoms[p.i].species.symbol;
                        let sym_j = s.atoms[p.j].species.symbol;
                        format!(
                            "closest pair {}{}-{}{}\n\
                             |  R         {:.4} bohr\n\
                             |  E_rel     {:+.4e} Eh\n\
                             |  r_outer   {:.4}\n\
                             |  bonded    {}",
                            sym_i,
                            p.i,
                            sym_j,
                            p.j,
                            p.r,
                            p.e_rel,
                            p.r_outer,
                            if p.bonded { "yes" } else { "no" }
                        )
                    }
                    None => "closest pair  (none)".to_string(),
                };
                text.0 = format!(
                    "preset     {}\n\
                     R_e        {:.5} bohr\n\
                     D_e        {:.6} Eh\n\
                     asymptote  {:.6} Eh\n\
                     knots      {}\n\
                     {pair}\n\
                     T          {:.1} K  ({} d.o.f./atom)\n\
                     Bonded means E_rel below the\n\
                     asymptote and R inside the turning\n\
                     point. No distance cutoff anywhere.",
                    world.preset.name(),
                    s.table().r_e,
                    s.table().d_e,
                    s.table().e_asymptote,
                    s.table().knots(),
                    s.temperature(),
                    s.dims.dof() as u32,
                );
            }
            Slot::CensusBody => {
                let c = &s.holons.census;
                text.0 = format!(
                    "micro . atoms          {}\n\
                     composite . molecules  {}\n\
                     candidate . evals/fr   {}\n\
                     global . views         {}\n\
                     formations             {}\n\
                     dissolutions           {}\n\
                     bound but not closed   {}\n\
                     bond-sector energy     {:+.3e}\n\
                     Being matter is expensive - the\n\
                     O(N*N) force loop is the whole\n\
                     budget. Being a holon is cheap.",
                    c.atoms,
                    c.molecules,
                    c.candidate_evaluations,
                    c.global_views,
                    c.formations,
                    c.dissolutions,
                    c.closure_rejections,
                    s.holons.bond_sector_energy(),
                );
            }
            Slot::DeviceBody => {
                let cal = match world.calibration {
                    Calibration::Pending => "measuring...",
                    Calibration::Done => "measured on load",
                    Calibration::Unavailable => "no clock; budget unlimited",
                };
                text.0 = format!(
                    "substeps / sec    {:.4e}\n\
                     pairs / sec       {:.4e}\n\
                     required s/sec    {:.4e}\n\
                     N_max here        {}\n\
                     calibration       {cal}\n\
                     Not a guess, and not this\n\
                     developer's machine.",
                    s.timescale.substeps_per_second,
                    world.pairs_per_second(),
                    s.timescale.required_substeps_per_second(),
                    world.n_max() as u64,
                );
            }
            Slot::LabelPreset => {
                text.0 = format!("preset: {}", world.preset.short_name());
            }
            Slot::LabelAtoms => text.0 = format!("+  ({} atoms)", s.n),
            Slot::LabelSpeed => {
                text.0 = format!("faster  ({:.3} fs/s)", s.timescale.sim_speed_fs_per_wallsec)
            }
            Slot::LabelWalls => {
                text.0 = match s.boundary {
                    Boundary::Walls => "walls ON".to_string(),
                    Boundary::Open => "walls OFF".to_string(),
                }
            }
            Slot::LabelThermostat => {
                text.0 = format!("thermostat {}", if s.thermostat_on { "ON" } else { "OFF" })
            }
            Slot::LabelCensus => {
                text.0 = format!("composite {}", if s.holons.enabled { "ON" } else { "OFF" })
            }
            Slot::LabelDtGrowth => {
                text.0 = if s.timescale.allow_dt_growth {
                    "dt GROWN (accuracy declared)".to_string()
                } else {
                    "dt held".to_string()
                }
            }
        }
    }
}

/// Slightly lifted green, so PASS reads against the dark panel rather than sinking into
/// it the way the scene's bond green would.
const GREEN_TEXT: Color = Color::srgb(0.44, 0.80, 0.64);

/// `measured / bound`, with the degenerate case named rather than printed as a NaN.
fn ratio(measured: f64, bound: f64) -> f64 {
    if bound > 0.0 {
        measured / bound
    } else {
        0.0
    }
}
