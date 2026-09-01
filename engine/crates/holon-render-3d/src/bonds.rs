//! Bonds, drawn as cylinders between the members of every bonded pair.
//!
//! A bond here is not a distance heuristic and not a drawn decoration. It is the pair
//! predicate's own answer, read off `PairReading::bonded`: the pair's relative energy is
//! below the dissociation asymptote AND its separation is inside the outer classical
//! turning point at that energy, both solved on the exact curve. So a cylinder appears
//! exactly when the physics says a molecule exists, and the census counts the same
//! events the picture shows.
//!
//! One entity per POSSIBLE pair, hidden unless that pair is bonded. `complete_pairs(DEFAULT_SCENE_ATOMS)` is 120,
//! which is cheap to keep resident and much cheaper than spawning and despawning
//! entities as bonds form and break — a bond can flicker at the dwell boundary, and
//! entity churn at frame rate is exactly what the persistent table avoids.

use bevy::prelude::*;
use holon_render::sim::{complete_pairs, DEFAULT_SCENE_ATOMS};

use crate::scene::{bond_radius, rod_between, to_world, SceneAssets};
use crate::world::AtomWorld;

/// One entity per possible pair, indexed by the pair index the simulation uses.
#[derive(Resource)]
pub struct BondEntities(pub Vec<Entity>);

pub fn setup_bonds(commands: &mut Commands, assets: &SceneAssets) {
    // T3: the engine's scene is no longer capped, so this is the VIEW's own budget — how
    // many bond cylinders the shell pre-creates for the opening scene — and not a statement
    // about how many bonds can exist. The view reads the pair list and never writes it, so
    // a larger scene draws the bonds it has entities for and the physics is unaffected.
    let budget = complete_pairs(DEFAULT_SCENE_ATOMS);
    let mut bonds = Vec::with_capacity(budget);
    for _ in 0..budget {
        bonds.push(
            commands
                .spawn((
                    Mesh3d(assets.rod_mesh.clone()),
                    MeshMaterial3d(assets.bond_mat.clone()),
                    Transform::default(),
                    Visibility::Hidden,
                ))
                .id(),
        );
    }
    commands.insert_resource(BondEntities(bonds));
}

pub fn sync_bonds(
    world: Res<AtomWorld>,
    bonds: Res<BondEntities>,
    mut transforms: Query<&mut Transform>,
    mut visibility: Query<&mut Visibility>,
) {
    let s = &world.sim;
    for (k, e) in bonds.0.iter().enumerate() {
        let show = k < s.pair_count && s.pairs[k].bonded;
        if let Ok(mut vis) = visibility.get_mut(*e) {
            *vis = if show {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
        if !show {
            continue;
        }
        let p = &s.pairs[k];
        let a = &s.atoms[p.i];
        let b = &s.atoms[p.j];
        if let Ok(mut tf) = transforms.get_mut(*e) {
            // The rod's thickness is the bond's DEPTH — |e_bond| as a fraction of
            // the well, exactly the encoding the 2D shell has always drawn. A pair
            // grazing the tail at 1e-4 Ha renders as a hair; a pair at the bottom
            // of the well renders as the full rod. Continuous and physical: no
            // threshold decides what is "really" a bond, the energy does — the
            // field report about entry-scene bonds is why this is not cosmetic.
            let depth = (-p.e_bond() / s.table().d_e.max(1e-12)).clamp(0.0, 1.0) as f32;
            *tf = rod_between(
                to_world(a.x, a.y, a.z),
                to_world(b.x, b.y, b.z),
                bond_radius() * (0.15 + 0.85 * depth),
            );
        }
    }
}
