//! A three-point rig plus an ambient term, sized to the box.
//!
//! Same shape as CIRISGame's `lighting.rs` — key / fill / rim, only the key casting
//! shadows so the webgl2 artifact keeps a single shadow map — with the tints taken from
//! the atom shell's own palette instead of the game's clay one.

use bevy::light::GlobalAmbientLight;
use bevy::prelude::*;

use crate::scene::PAPER;
use crate::world::BOX_SIDE;

/// Base illuminance (lux) the multipliers below scale.
const BASE_LUX: f32 = 9_000.0;

pub fn spawn_rig(commands: &mut Commands) {
    // Light positions scale with the box, so a bigger box is not a darker one.
    let scale = BOX_SIDE as f32 / 24.0;

    // Key — warm, upper-right-front. The only shadow caster.
    spawn_dir(
        commands,
        scale,
        Vec3::new(16.0, 22.0, 12.0),
        Color::srgb(1.0, 0.96, 0.90),
        BASE_LUX * 1.1,
        true,
    );
    // Fill — cool, lower-left, half strength.
    spawn_dir(
        commands,
        scale,
        Vec3::new(-14.0, 6.0, 10.0),
        Color::srgb(0.86, 0.90, 0.94),
        BASE_LUX * 0.5,
        false,
    );
    // Rim — a rake from behind, so a sphere against the dark background keeps an edge.
    spawn_dir(
        commands,
        scale,
        Vec3::new(2.0, 9.0, -18.0),
        Color::srgb(0.72, 0.92, 0.84),
        BASE_LUX * 1.15,
        false,
    );

    // Low ambient against the ink background: enough that an unlit face is not black,
    // little enough that the emissive bonds still read as the brightest thing present.
    commands.insert_resource(GlobalAmbientLight {
        color: PAPER,
        brightness: 60.0,
        ..default()
    });
}

fn spawn_dir(
    commands: &mut Commands,
    scale: f32,
    pos: Vec3,
    color: Color,
    lux: f32,
    shadows: bool,
) {
    commands.spawn((
        DirectionalLight {
            color,
            illuminance: lux,
            shadow_maps_enabled: shadows,
            ..default()
        },
        Transform::from_translation(pos * scale).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
