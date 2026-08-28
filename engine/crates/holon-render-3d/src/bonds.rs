//! Bonds, drawn as cylinders between the members of every bonded pair.
//!
//! A bond here is not a distance heuristic and not a drawn decoration. It is the pair
//! predicate's own answer, read off `PairReading::bonded`: the pair's relative energy is
//! below the dissociation asymptote AND its separation is inside the outer classical
//! turning point at that energy, both solved on the exact curve. So a cylinder appears
//! exactly when the physics says a molecule exists, and the census counts the same
//! events the picture shows.
//!
//! One entity per POSSIBLE pair, hidden unless that pair is bonded. `MAX_PAIRS` is 120,
//! which is cheap to keep resident and much cheaper than spawning and despawning
//! entities as bonds form and break — a bond can flicker at the dwell boundary, and
//! entity churn at frame rate is exactly what the persistent table avoids.

use bevy::prelude::*;
use holon_render::sim::MAX_PAIRS;

use crate::scene::{bond_radius, rod_between, to_world, SceneAssets};
use crate::world::AtomWorld;

/// One entity per possible pair, indexed by the pair index the simulation uses.
#[derive(Resource)]
pub struct BondEntities(pub Vec<Entity>);

pub fn setup_bonds(commands: &mut Commands, assets: &SceneAssets) {
    let mut bonds = Vec::with_capacity(MAX_PAIRS);
    for _ in 0..MAX_PAIRS {
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
            *tf = rod_between(
                to_world(a.x, a.y, a.z),
                to_world(b.x, b.y, b.z),
                bond_radius(),
            );
        }
    }
}
