//! The Bevy app: camera, plugin wiring, and the per-frame advance.
//!
//! Feature-gated behind `render`; never compiled into the headless build.
//!
//! System order is chained and fixed, because every stage reads the one below it and
//! must see a state nothing above it has modified:
//!
//! ```text
//! calibrate → drag_atom → apply_hand_intent → advance_world
//!           → fill_frame_from_world → sync_atoms → sync_bonds → update_hud
//! ```
//!
//! The two Route B waists sit at the two ends of that line. `drag_atom` READS the buffer
//! filled at the end of the previous frame and WRITES the hand's intent; the sink applies
//! that intent to this page's `Sim` before the step, so a gesture and the step that
//! carries it out land in one frame. Everything between the sink and the fill is physics
//! and everything after it is drawing, which is what lets the workbench replace exactly
//! two systems — the sink and the fill — and change nothing else.
//!
//! `advance_world` is the ONLY system that steps the physics, and it steps it exactly
//! once per rendered frame with the interval the frame actually took. Nothing here
//! assumes 60 Hz, or any Hz.

// HDR is withheld from the webgl2 artifact (bevy #7352 — see `setup`), so the import is
// gated the same way the component is.
#[cfg(not(feature = "webgl2"))]
use bevy::camera::Hdr;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use crate::scene::{self, INK};
use crate::world::{AtomWorld, Calibration, BOX_SIDE, CALIBRATION_SUBSTEPS};
use crate::{bonds, hud, lighting, pick, OPENING_ATOMS};

/// The 3D camera. The picker and the drag suppression target this one specifically.
#[derive(Component)]
pub struct MainCam;

pub fn run_app() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "CIRISHolon · atom world".into(),
            // wasm: bind to the <canvas id="atoms3d"> in docs/atoms3d/index.html and let
            // it track the parent element.
            canvas: Some("#atoms3d".into()),
            fit_canvas_to_parent: true,
            prevent_default_event_handling: true,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(PanOrbitCameraPlugin)
    .add_plugins(pick::plugin)
    .add_plugins(hud::plugin)
    .insert_resource(ClearColor(INK))
    .insert_resource(AtomWorld::new(OPENING_ATOMS))
    .init_resource::<crate::frame::FrameBuffer>()
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        (
            calibrate,
            // The finger, then what the finger asked for. Both ahead of the step, so a
            // grab takes hold of the atom in the frame the user pressed on it.
            pick::drag_atom,
            apply_hand_intent,
            advance_world,
            // PRODUCE, then draw. The buffer is filled from this page's own Sim between
            // stepping and drawing, so the renderer never sees a Sim and the ordering
            // makes "the drawn scene is the stepped scene" a schedule property rather
            // than a hope. The workbench's producer sits in this slot instead, filling
            // the same type from the committed cdylib.
            fill_frame_from_world,
            scene::sync_atoms,
            bonds::sync_bonds,
        )
            .chain(),
    );

    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let side = BOX_SIDE as f32;

    let mut camera = commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            ..default()
        },
        // Msaa::Off unconditionally: HDR Rgba16Float cannot be multisampled, and the
        // webgl2 path has its own reasons below.
        Msaa::Off,
        Tonemapping::AgX,
        PanOrbitCamera {
            // Far enough out that the box diagonal (side * sqrt 3) fits the default
            // vertical field of view with room to spare.
            radius: Some(side * 2.1),
            yaw: Some(std::f32::consts::FRAC_PI_4),
            pitch: Some(0.42),
            // Close enough to sit between two bonded atoms; far enough to see the whole
            // box from outside.
            zoom_lower_limit: 1.0,
            zoom_upper_limit: Some(side * 6.0),
            ..default()
        },
        MainCam,
    ));

    // HDR and Bloom are BOTH withheld from the webgl2 artifact, and that is a deliberate
    // difference between the two wasm builds rather than an oversight.
    //
    // WebGL2 cannot do HDR in the browser (bevy #7352): a camera marked `Hdr` on that
    // backend renders magenta, which is not a degraded look, it is no look at all. And
    // bloom without HDR has nothing above 1.0 to bloom. So webgl2 — the phone-compatible
    // fallback — gets the same geometry, the same physics and the same overlay in plain
    // LDR, and webgpu gets the glow. The one thing neither build may do is disagree
    // about a number.
    #[cfg(not(feature = "webgl2"))]
    {
        camera.insert(Hdr);
        #[cfg(feature = "bloom")]
        camera.insert(bevy::post_process::bloom::Bloom::NATURAL);
    }
    // `camera` is consumed above only under cfg; bind it so both paths compile.
    let _ = &mut camera;

    // A separate 2D camera owns the UI, compositing on top of the 3D without clearing
    // it. Its HDR-ness MUST match the 3D camera's: a mix of HDR and non-HDR cameras
    // renders magenta on Metal (bevy #6754).
    let mut ui_camera = commands.spawn((
        Camera2d,
        Msaa::Off,
        Camera {
            order: 2,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        IsDefaultUiCamera,
    ));
    #[cfg(not(feature = "webgl2"))]
    ui_camera.insert(Hdr);
    let _ = &mut ui_camera;

    lighting::spawn_rig(&mut commands);
    scene::setup_scene(&mut commands, &mut meshes, &mut materials);
}

/// Measure this device, once, on a frame of its own.
///
/// The burst is PURE PHYSICS at the calibration scene — no grain closure, no rendering —
/// and the host times it with its own clock. That measurement is the authority for
/// `N_max`: an atom count the device cannot carry would otherwise be delivered as
/// silent time dilation, which is a worse answer than saying so.
fn calibrate(mut world: ResMut<AtomWorld>) {
    if world.calibration != Calibration::Pending || !world.table_ok() {
        return;
    }
    let started = bevy::platform::time::Instant::now();
    world.calibration_burst(CALIBRATION_SUBSTEPS);
    let seconds = started.elapsed().as_secs_f64();
    if seconds > 0.0 {
        world.record_calibration(CALIBRATION_SUBSTEPS as f64 / seconds);
    } else {
        // A clock with no resolution cannot measure a burst. Reported, not invented.
        world.record_calibration(f64::NAN);
    }
}

/// Advance one frame of MEASURED wall time.
/// atoms3d's producer: fill the frame buffer from the `AtomWorld` this page owns.
///
/// One line, because `AtomWorld::fill_frame` owns the mapping and this only says WHEN.
/// The workbench replaces this system and nothing else — same buffer, same consumers, a
/// different source — which is the whole of Route B's "one consumer, two producers".
fn fill_frame_from_world(world: Res<AtomWorld>, mut frame: ResMut<crate::frame::FrameBuffer>) {
    world.fill_frame(&mut frame);
}

/// atoms3d's sink: apply the hand's intent to the `Sim` this page owns.
///
/// The mirror of `fill_frame_from_world`, and the other system the workbench replaces —
/// there the same queue is drained by JS onto the cdylib's `holon_grab` /
/// `holon_move_anchor_3d` / `holon_release`. One line here for the same reason: the
/// mapping from op to door lives in `HandIntent::apply_to`, stated once, so the two pages
/// cannot disagree about what a drag does to the ledger.
fn apply_hand_intent(mut hand: ResMut<crate::hand::HandIntent>, mut world: ResMut<AtomWorld>) {
    hand.apply_to(&mut world.sim);
}

fn advance_world(time: Res<Time>, mut world: ResMut<AtomWorld>) {
    // The calibration frame is not a physics frame: its interval is dominated by the
    // burst, and spending it as sim-time would produce a visible jump on the first
    // frame after load.
    if world.calibration == Calibration::Pending {
        return;
    }
    let dt = time.delta_secs_f64();
    world.advance(dt);
}
