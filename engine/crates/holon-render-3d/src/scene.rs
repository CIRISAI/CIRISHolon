//! Geometry: the box, the atoms, and the mapping between the simulation's box and the
//! world the camera looks at.
//!
//! One mesh handle and one material handle are shared by every atom, so Bevy batches the
//! spheres into instanced draws rather than issuing one per atom. The material is only
//! ever *swapped* when an atom's state actually changes (free / bonded / held), for the
//! reason `CIRISGame`'s `sync_board` gives: re-inserting a material on every entity every
//! frame re-prepares it in the render world, which reads as a redraw flicker.
//!
//! The simulation's box runs `[0, BOX_SIDE]` on each axis; the camera looks at a box
//! centred on the origin. [`to_world`] and [`to_sim`] are the only two places that know
//! that, and everything else stays in one frame or the other.

use bevy::prelude::*;
use holon_render::sim::MAX_ATOMS;

use crate::world::{AtomWorld, ATOM_RADIUS, BOX_SIDE};

// ── palette (docs/atoms/styles.css, so the two shells are one identity) ──────────
/// `--ink`. The background of a volume rather than a page: the 2D shell is dark type on
/// paper, and the 3D one inverts that so the lit spheres have something to read against.
pub const INK: Color = Color::srgb(0.090, 0.125, 0.114);
/// `--paper`.
pub const PAPER: Color = Color::srgb(0.929, 0.914, 0.874);
/// `--muted`.
pub const MUTED: Color = Color::srgb(0.412, 0.451, 0.431);
/// `--green`: an atom in a bonded pair, and the bond itself.
pub const GREEN: Color = Color::srgb(0.137, 0.412, 0.341);
/// `--green-soft`: a free atom.
pub const GREEN_SOFT: Color = Color::srgb(0.439, 0.651, 0.557);
/// `--rust`: the user's spring, and a failed gate.
pub const RUST: Color = Color::srgb(0.733, 0.298, 0.180);
/// `--amber`: the atom in your hand.
pub const AMBER: Color = Color::srgb(0.831, 0.604, 0.212);

/// Bond cylinder radius, bohr.
const BOND_RADIUS: f32 = 0.14;
/// Spring line radius, bohr. Thinner than a bond on purpose: the hand is not a bond.
const SPRING_RADIUS: f32 = 0.06;
/// Box edge bar thickness, bohr.
const EDGE_THICKNESS: f32 = 0.06;

/// Simulation coordinates (bohr, box at `[0, BOX_SIDE]`) → world (box centred on origin).
pub fn to_world(x: f64, y: f64, z: f64) -> Vec3 {
    let half = 0.5 * BOX_SIDE;
    Vec3::new((x - half) as f32, (y - half) as f32, (z - half) as f32)
}

/// World → simulation coordinates. The inverse of [`to_world`], and the only other place
/// that knows about the offset.
pub fn to_sim(v: Vec3) -> (f64, f64, f64) {
    let half = 0.5 * BOX_SIDE;
    (v.x as f64 + half, v.y as f64 + half, v.z as f64 + half)
}

/// Shared handles, built once at startup and cloned per entity.
#[derive(Resource)]
pub struct SceneAssets {
    pub atom_mesh: Handle<Mesh>,
    /// Unit cylinder (radius 1, height 1, along +Y). Scaled per bond and per spring.
    pub rod_mesh: Handle<Mesh>,
    /// Unit cube. Scaled per box edge.
    pub edge_mesh: Handle<Mesh>,
    pub free_mat: Handle<StandardMaterial>,
    pub bonded_mat: Handle<StandardMaterial>,
    pub held_mat: Handle<StandardMaterial>,
    pub bond_mat: Handle<StandardMaterial>,
    pub spring_mat: Handle<StandardMaterial>,
    pub edge_mat: Handle<StandardMaterial>,
}

/// How an atom is being drawn. Tracked so the material is only swapped on a change.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AtomLook {
    Free,
    Bonded,
    Held,
}

/// The per-atom entity table, indexed by atom index. Entities for atoms above the
/// current count exist but are hidden — the count changes at runtime and rebuilding the
/// table would churn the render world for no reason.
#[derive(Resource)]
pub struct AtomEntities {
    pub atoms: Vec<Entity>,
    pub look: Vec<AtomLook>,
    /// The rod drawn from the held atom to the anchor, and the little marker on the
    /// anchor itself. Hidden unless something is grabbed.
    pub spring: Entity,
    pub anchor: Entity,
}

pub fn setup_scene(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let assets = SceneAssets {
        atom_mesh: meshes.add(Sphere::new(ATOM_RADIUS as f32).mesh().ico(4).unwrap()),
        rod_mesh: meshes.add(Cylinder::new(1.0, 1.0)),
        edge_mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        free_mat: materials.add(atom_material(GREEN_SOFT, 0.0)),
        bonded_mat: materials.add(atom_material(GREEN, 0.35)),
        held_mat: materials.add(atom_material(AMBER, 0.9)),
        bond_mat: materials.add(atom_material(GREEN, 1.4)),
        spring_mat: materials.add(atom_material(RUST, 0.8)),
        edge_mat: materials.add(StandardMaterial {
            base_color: MUTED,
            perceptual_roughness: 0.9,
            metallic: 0.0,
            unlit: true,
            ..default()
        }),
    };

    // One persistent entity per atom SLOT, not per live atom.
    let mut atoms = Vec::with_capacity(MAX_ATOMS);
    for _ in 0..MAX_ATOMS {
        atoms.push(
            commands
                .spawn((
                    Mesh3d(assets.atom_mesh.clone()),
                    MeshMaterial3d(assets.free_mat.clone()),
                    Transform::default(),
                    Visibility::Hidden,
                ))
                .id(),
        );
    }

    let spring = commands
        .spawn((
            Mesh3d(assets.rod_mesh.clone()),
            MeshMaterial3d(assets.spring_mat.clone()),
            Transform::default(),
            Visibility::Hidden,
        ))
        .id();
    let anchor = commands
        .spawn((
            Mesh3d(assets.atom_mesh.clone()),
            MeshMaterial3d(assets.spring_mat.clone()),
            Transform::from_scale(Vec3::splat(0.25)),
            Visibility::Hidden,
        ))
        .id();

    spawn_box_edges(commands, &assets);
    // Built here rather than in a later system because the bond entities need these
    // handles, and a resource inserted through `Commands` is not visible until the next
    // flush — so a second startup system would have to wait a frame for them.
    crate::bonds::setup_bonds(commands, &assets);

    commands.insert_resource(AtomEntities {
        atoms,
        look: vec![AtomLook::Free; MAX_ATOMS],
        spring,
        anchor,
    });
    commands.insert_resource(assets);
}

fn atom_material(color: Color, emissive: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        emissive: LinearRgba::from(color) * emissive,
        perceptual_roughness: 0.35,
        metallic: 0.05,
        ..default()
    }
}

/// The twelve edges of the box, as thin bars. A wireframe rather than a translucent
/// solid: the box is the wall potential's location, and it has to be readable from
/// inside as well as outside without sorting six transparent faces every frame.
fn spawn_box_edges(commands: &mut Commands, assets: &SceneAssets) {
    let h = 0.5 * BOX_SIDE as f32;
    let t = EDGE_THICKNESS;
    let s = BOX_SIDE as f32;
    for &(sy, sz) in &[(-1.0f32, -1.0f32), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)] {
        // Along x, then y, then z: the same four-corner pattern rotated.
        commands.spawn((
            Mesh3d(assets.edge_mesh.clone()),
            MeshMaterial3d(assets.edge_mat.clone()),
            Transform::from_translation(Vec3::new(0.0, sy * h, sz * h))
                .with_scale(Vec3::new(s, t, t)),
        ));
        commands.spawn((
            Mesh3d(assets.edge_mesh.clone()),
            MeshMaterial3d(assets.edge_mat.clone()),
            Transform::from_translation(Vec3::new(sy * h, 0.0, sz * h))
                .with_scale(Vec3::new(t, s, t)),
        ));
        commands.spawn((
            Mesh3d(assets.edge_mesh.clone()),
            MeshMaterial3d(assets.edge_mat.clone()),
            Transform::from_translation(Vec3::new(sy * h, sz * h, 0.0))
                .with_scale(Vec3::new(t, t, s)),
        ));
    }
}

/// Move every atom entity to where the simulation says its atom is, show the live ones,
/// hide the rest, and repaint only the ones whose state changed.
pub fn sync_atoms(
    world: Res<AtomWorld>,
    assets: Res<SceneAssets>,
    mut table: ResMut<AtomEntities>,
    mut commands: Commands,
    mut transforms: Query<&mut Transform>,
    mut visibility: Query<&mut Visibility>,
) {
    let s = &world.sim;
    // An atom is drawn as bonded when it is a member of ANY bonded pair. That is the
    // pair predicate's own reading, not a distance test of this module's invention.
    let mut bonded = [false; MAX_ATOMS];
    for p in &s.pairs[..s.pair_count] {
        if p.bonded {
            bonded[p.i] = true;
            bonded[p.j] = true;
        }
    }

    for i in 0..MAX_ATOMS {
        let e = table.atoms[i];
        let live = i < s.n;
        if let Ok(mut vis) = visibility.get_mut(e) {
            *vis = if live {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
        if !live {
            continue;
        }
        let a = &s.atoms[i];
        if let Ok(mut tf) = transforms.get_mut(e) {
            tf.translation = to_world(a.x, a.y, a.z);
        }
        let look = if s.grabbed == Some(i) {
            AtomLook::Held
        } else if bonded[i] {
            AtomLook::Bonded
        } else {
            AtomLook::Free
        };
        if look != table.look[i] {
            let mat = match look {
                AtomLook::Free => assets.free_mat.clone(),
                AtomLook::Bonded => assets.bonded_mat.clone(),
                AtomLook::Held => assets.held_mat.clone(),
            };
            commands.entity(e).insert(MeshMaterial3d(mat));
            table.look[i] = look;
        }
    }

    // The hand: a thin rod from the held atom to the anchor, and a marker on the anchor.
    // Drawn because the spring is a term in the ledger — if the user can see W_ext move,
    // they should be able to see what moved it.
    let held = s.grabbed.filter(|&i| i < s.n);
    let show = held.is_some();
    for e in [table.spring, table.anchor] {
        if let Ok(mut vis) = visibility.get_mut(e) {
            *vis = if show {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
    if let Some(i) = held {
        let a = &s.atoms[i];
        let from = to_world(a.x, a.y, a.z);
        let to = to_world(s.anchor.0, s.anchor.1, s.anchor.2);
        if let Ok(mut tf) = transforms.get_mut(table.spring) {
            *tf = rod_between(from, to, SPRING_RADIUS);
        }
        if let Ok(mut tf) = transforms.get_mut(table.anchor) {
            tf.translation = to;
        }
    }
}

/// The transform that places the unit cylinder (radius 1, height 1, along +Y) as a rod
/// of radius `radius` spanning `from` to `to`.
pub fn rod_between(from: Vec3, to: Vec3, radius: f32) -> Transform {
    let delta = to - from;
    let length = delta.length();
    let rotation = if length > 1.0e-6 {
        Quat::from_rotation_arc(Vec3::Y, delta / length)
    } else {
        Quat::IDENTITY
    };
    Transform {
        translation: from + 0.5 * delta,
        rotation,
        scale: Vec3::new(radius, length.max(1.0e-6), radius),
    }
}

/// Bond rod radius, exported so `bonds.rs` uses the same number the palette was chosen
/// against rather than a second copy of it.
pub const fn bond_radius() -> f32 {
    BOND_RADIUS
}
