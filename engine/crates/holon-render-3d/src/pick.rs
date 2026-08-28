//! Dragging an atom with a pointer or a finger.
//!
//! The interaction is the SAME one the canvas shell has, and deliberately so: the
//! pointer does not move an atom, it moves the anchor of a finite spring whose energy is
//! a term in the Hamiltonian, and the work that anchor motion does is posted to `W_ext`
//! in the same breath. That is why `E - W_ext` stays constant through a drag, and it is
//! why the hand can be used as a *brake* to make H2 — the only channel, in a two-atom
//! scene, by which a bond can form at all.
//!
//! Nothing in this module computes work, energy or force. It computes a point in space
//! and hands it to `Sim::move_anchor_3d`.
//!
//! # Three dimensions, one screen
//!
//! A pointer names a ray, not a point, so the drag needs a surface. The surface is the
//! plane through the grabbed atom with the CAMERA'S FORWARD as its normal — the
//! screen-parallel plane through the atom, frozen at the moment of the grab. Frozen
//! rather than re-derived per frame, because a plane that follows the camera would make
//! an orbit into a drag; and screen-parallel rather than axis-aligned, because the
//! natural reading of a drag is "the atom follows my finger", which is exactly what a
//! screen-parallel plane delivers from any viewing angle. To move an atom along the view
//! axis, orbit first and then drag: the depth you cannot reach is the depth you cannot
//! see.
//!
//! # Pointer and touch
//!
//! Both, through one path. A mouse press and a single touch produce the same
//! `(position, began, ended)`; two or more simultaneous touches are a camera gesture
//! (`bevy_panorbit_camera`'s pinch, and the fly-through in `render.rs`) and never a
//! drag, so a two-finger zoom cannot pick up an atom by accident. While a drag is live
//! the orbit camera is suppressed, the same way CIRISGame suppresses it during a UI
//! drag, so one gesture never means two things.

use bevy::input::touch::Touches;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::render::MainCam;
use crate::scene::{to_sim, to_world};
use crate::world::{AtomWorld, ATOM_RADIUS};

/// Angular slop on the pick, radians. The perpendicular-distance threshold grows with
/// range as `t * PICK_SLOP`, so the pick is a fixed angle on screen rather than a fixed
/// distance in the world: generous for a fingertip on a phone at a wide view, and still
/// tight enough to separate two bonded atoms 1.4 bohr apart when zoomed in on them.
const PICK_SLOP: f32 = 0.022;

/// Which pointer, if any, currently owns a drag.
#[derive(Resource, Default)]
pub struct DragState {
    /// The touch id driving the drag, or `None` when the mouse is.
    touch: Option<u64>,
    active: bool,
    /// The screen-parallel plane through the grabbed atom, frozen at the grab.
    plane_point: Vec3,
    plane_normal: Vec3,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<DragState>()
        .add_systems(Update, drag_atom);
}

/// One frame of the drag: resolve the pointer, pick on press, follow on move, let go on
/// release.
#[allow(clippy::too_many_arguments)]
fn drag_atom(
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform), With<MainCam>>,
    buttons: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    ui_buttons: Query<&Interaction, With<Button>>,
    mut drag: ResMut<DragState>,
    mut world: ResMut<AtomWorld>,
    mut orbit: Query<&mut PanOrbitCamera, With<MainCam>>,
) {
    // A press that lands on a HUD button belongs to the HUD.
    let over_ui = ui_buttons
        .iter()
        .any(|i| matches!(i, Interaction::Pressed | Interaction::Hovered));

    let touch_count = touches.iter().count();
    // Two or more fingers is a camera gesture; a live drag is abandoned rather than
    // fought over.
    let multi_touch = touch_count >= 2;

    // ── resolve one pointer ────────────────────────────────────────────────
    let (position, began, ended) = if let Some(id) = drag.touch {
        // A touch drag is in progress: follow that finger and no other.
        match touches.get_pressed(id) {
            Some(t) if !multi_touch => (Some(t.position()), false, false),
            _ => (None, false, true),
        }
    } else if touch_count == 1 {
        let t = touches.iter().next();
        let just = touches.iter_just_pressed().next().map(|t| t.id());
        (t.map(|t| t.position()), just.is_some(), false)
    } else if multi_touch {
        (None, false, drag.active)
    } else {
        let cursor = windows.single().ok().and_then(|w| w.cursor_position());
        let down = buttons.pressed(MouseButton::Left);
        (
            cursor.filter(|_| down),
            buttons.just_pressed(MouseButton::Left),
            buttons.just_released(MouseButton::Left),
        )
    };

    if ended || (drag.active && position.is_none()) {
        if drag.active {
            // Release subtracts the energy still stored in the spring, because it leaves
            // the scene with the hand. `Sim::release` does that; this module only says
            // when.
            world.sim.release();
        }
        drag.active = false;
        drag.touch = None;
        if let Ok(mut cam) = orbit.single_mut() {
            cam.enabled = true;
        }
        return;
    }

    let Some(position) = position else {
        return;
    };
    let Ok((camera, cam_tf)) = cameras.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(cam_tf, position) else {
        return;
    };
    let dir = ray.direction.as_vec3();

    // ── the grab ───────────────────────────────────────────────────────────
    if began && !drag.active && !over_ui {
        let mut best: Option<(f32, usize, Vec3)> = None;
        for i in 0..world.sim.n {
            let a = &world.sim.atoms[i];
            let centre = to_world(a.x, a.y, a.z);
            let t = (centre - ray.origin).dot(dir);
            if t <= 0.0 {
                continue;
            }
            let perp = (ray.origin + dir * t).distance(centre);
            // Angular tolerance, floored at the drawn radius so a very near atom is
            // still pickable at its own size.
            let tolerance = (t * PICK_SLOP).max(ATOM_RADIUS as f32 * 1.2);
            if perp < tolerance && best.is_none_or(|(bt, _, _)| t < bt) {
                best = Some((t, i, centre));
            }
        }
        if let Some((_, i, centre)) = best {
            // The grab itself injects nothing: `Sim::grab` puts the anchor ON the atom,
            // so the spring enters the ledger at zero extension.
            world.sim.grab(i);
            drag.active = true;
            drag.plane_point = centre;
            // Frozen at the grab — see the module header on why this is not re-derived.
            drag.plane_normal = cam_tf.forward().as_vec3();
            drag.touch = touches.iter_just_pressed().next().map(|t| t.id());
            if let Ok(mut cam) = orbit.single_mut() {
                cam.enabled = false;
            }
        }
        return;
    }

    // ── the follow ─────────────────────────────────────────────────────────
    if !drag.active {
        return;
    }
    let denominator = dir.dot(drag.plane_normal);
    // A ray parallel to the plane has no intersection to move to; the anchor simply
    // stays where it was rather than being sent somewhere arbitrary.
    if denominator.abs() < 1.0e-6 {
        return;
    }
    let t = (drag.plane_point - ray.origin).dot(drag.plane_normal) / denominator;
    if t <= 0.0 {
        return;
    }
    let hit = ray.origin + dir * t;
    let (x, y, z) = to_sim(hit);
    world.sim.move_anchor_3d(x, y, z);
}
