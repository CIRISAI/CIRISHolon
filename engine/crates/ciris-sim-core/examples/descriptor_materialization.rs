//! Descriptor-as-generator materialization: a one-million-constituent wall bound to a
//! stone descriptor holon decomposes into grains that carry mineral identity, size,
//! and quenched flaw strength — drawn deterministically from the descriptor's declared
//! distributions and certified by the statistical-composition gate.
//!
//! Run with:
//! ```sh
//! cargo run -p ciris-sim-core --example descriptor_materialization --features alloc
//! ```

use ciris_sim_core::descriptor::{
    build_stone_descriptor, certify_grains, draw_grains, encode_seed, read_grain,
    DescriptorMaterializer, DrawParams,
};
use ciris_sim_core::holon::{Channels, Decomposition};
use ciris_sim_core::material::{IsotropicMaterial, MaterialBinding};
use ciris_sim_core::regplus::GrossState;
use ciris_sim_core::runtime::{RuntimeArena, RuntimeHolonSpec, NO_RUNTIME_HOLON};

const MINERALS: [&str; 3] = ["quartz", "feldspar", "mica"];

fn wall(seed: u64) -> RuntimeArena {
    let whole = encode_seed(seed);
    let specs = [RuntimeHolonSpec {
        parent: NO_RUNTIME_HOLON,
        depth: 0,
        grain_units: 8,
        gross: GrossState::aggregate(1_000_000, 2_000_000, [1_000_000, -3]),
        whole: &whole,
        channels: Channels::REG_PLUS.union(Channels::MECHANICAL),
        boundary: true,
        decomposition: Decomposition::Latent,
    }];
    RuntimeArena::from_specs(&specs, 0).unwrap()
}

fn expand_fully(materializer: &mut DescriptorMaterializer<'_>, arena: &mut RuntimeArena) {
    let mut i = 0;
    while i < arena.len() {
        if arena.holon(i).unwrap().decomposition == Decomposition::Latent {
            materializer
                .materialize_described(arena, i)
                .expect("honest generator must pass its own certificate");
        }
        i += 1;
    }
}

fn main() {
    // The stone descriptor is an ordinary holon: modal fractions are its mineral
    // children's exact gross ledgers, the ensemble distributions are its whole-only
    // state. Demo values: Westerly-class d50 = 0.75 mm, grain-scale weakest-link
    // Weibull (m = 10, sigma_0 = 200 MPa, lambda = 1e10 flaws/m^3).
    let law = DrawParams {
        grain_mu_ln_m: (7.5e-4_f64).ln(),
        grain_sigma_ln: 0.5,
        weibull_m: 10.0,
        weibull_sigma0_pa: 2.0e8,
        flaw_density_per_m3: 1.0e10,
    };
    let descriptors = build_stone_descriptor(1_000_000, &[30, 60, 10], &law).unwrap();
    println!("descriptor holarchy: {} holons; mineral ledgers:", descriptors.len());
    for (name, id) in MINERALS.iter().zip(1..descriptors.len()) {
        println!(
            "  {:8} {:>7} constituents",
            name,
            descriptors.holon(id).unwrap().gross.constituents
        );
    }

    let binding = MaterialBinding {
        subject_holon: 0,
        descriptor_holon: 0,
        properties: IsotropicMaterial::DEMO_CALIBRATION,
    };
    let mut materializer = DescriptorMaterializer::new(&descriptors, binding, 8, 0).unwrap();

    let mut arena = wall(0xC1F1_5000_0000_0042);
    expand_fully(&mut materializer, &mut arena);
    arena.validate().unwrap();

    let grains: Vec<_> = (0..arena.len())
        .filter(|i| arena.holon(*i).unwrap().decomposition == Decomposition::Leaf)
        .map(|i| read_grain(&arena, i).unwrap())
        .collect();
    println!(
        "\nwall decomposed: {} resident holons, {} leaf grains",
        arena.len(),
        grains.len()
    );
    let mut counts = [0_usize; 3];
    let mut diameter_sum = 0.0;
    for grain in &grains {
        counts[grain.mineral] += 1;
        diameter_sum += grain.diameter_m;
    }
    let (declared, fractions) = materializer.declared();
    for (k, name) in MINERALS.iter().enumerate() {
        println!(
            "  {:8} empirical {:.4} declared {:.4}",
            name,
            counts[k] as f64 / grains.len() as f64,
            fractions[k]
        );
    }
    println!(
        "  mean grain diameter {:.3} mm (declared median {:.3} mm)",
        1e3 * diameter_sum / grains.len() as f64,
        1e3 * declared.grain_mu_ln_m.exp()
    );

    let report = certify_grains(&grains, declared, fractions);
    println!(
        "  statistical-composition certificate: n={} passed={} unresolved={} rejected={}",
        report.n,
        report.passed,
        report.unresolved,
        report.rejected()
    );
    assert!(!report.rejected());

    // Quenched realization: the same wall (same persisted Record seed) decomposes to
    // bit-identical children on replay.
    let mut replay = wall(0xC1F1_5000_0000_0042);
    let mut replay_materializer = DescriptorMaterializer::new(&descriptors, binding, 8, 0).unwrap();
    expand_fully(&mut replay_materializer, &mut replay);
    let identical = arena.holons() == replay.holons()
        && arena
            .whole_scalars()
            .iter()
            .zip(replay.whole_scalars())
            .all(|(a, b)| a.to_bits() == b.to_bits());
    println!("\nreplay with the persisted seed is bit-identical: {identical}");
    assert!(identical);

    // The gate has teeth, scaled to what n can resolve: a generator drawing from a
    // wrong Weibull modulus is caught at n = 512 and declared unresolvable at n = 6.
    let mut mutant = law;
    mutant.weibull_m *= 3.8;
    let fired = certify_grains(&draw_grains(7, &mutant, fractions, 512), declared, fractions);
    let unresolved = certify_grains(&draw_grains(7, &mutant, fractions, 6), declared, fractions);
    println!(
        "planted wrong-Weibull generator: rejected at n=512: {:?}; at n=6: rejected={} unresolved checks={}",
        fired.failed,
        unresolved.rejected(),
        unresolved.unresolved
    );
    assert!(fired.rejected() && !unresolved.rejected());
}
