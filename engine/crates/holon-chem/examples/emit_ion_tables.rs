//! Emit the charged tables — and the REFUSALS, which are part of the bank.
//!
//! ```text
//! cargo run --release -p holon-chem --example emit_ion_tables -- [out_dir]
//! ```
//!
//! # What this list is, and what it is not
//!
//! The species below are the artifacts THIS CAMPAIGN produced, staked in
//! `conformance/atomworld/ION_TABLES_PREREG.md` before the generator existed. They are not
//! a registry of "the ions we support": `ION_STAKING.md` forbids one, and rightly — a
//! charged fragment is a species list, a geometry and an integer, and a lookup table of
//! blessed ions would be the table-lookup shape this crate refuses everywhere else. Adding
//! a species here costs a staked cut and a channel enumeration, which is exactly what it
//! should cost.
//!
//! # The refusals are emitted too
//!
//! A bank that simply omits what it will not serve teaches a consumer nothing: the absence
//! of `OH-.json` looks identical to a file nobody got round to making. So the manifest
//! carries a `refused` array in which every entry names its refusal, its fence and its
//! exit. `OBJECT.md`'s law 9 — refusal is a feature, and it names the gate whose passing
//! would lift it — applied to the published artifact rather than only to the code.
//!
//! # Two uncertainties, never one
//!
//! Every row carries `solver_exit` beside `uncertainty_hartree` beside
//! `solver_budget_iterations`, and `interpolant_uncertainty_hartree` beside those. The
//! first three go together because a capped residual is not monotone in solver effort: a
//! residual without its budget is a number whose meaning is missing. The fourth is a
//! different quantity entirely — the residual describes the SOLVE, the interpolant error
//! describes the GRID, and a consumer integrating the curve is exposed to the second one.
//! The sweep here is the FULL one, over every interval of the grid; `tests/ion_tables.rs`
//! gates a staked subset of it.

use holon_chem::elements::{HYDROGEN, OXYGEN};
use holon_chem::ion_table::{
    generate_ion_table, interpolant_error, Channel, Fragment, IonTable, StretchCut, TableRefusal,
};
use std::time::Instant;

/// The staked domain and grid, from the freeze.
const Q_MIN: f64 = 1.0;
const Q_MAX: f64 = 12.0;
const KNOTS: usize = 96;

/// Node C's staked pyramid, unchanged.
const R_OH_H3O: f64 = 1.85;
const ANGLE_H3O_DEG: f64 = 113.0;

fn cone() -> (f64, f64) {
    let cos_hoh = ANGLE_H3O_DEG.to_radians().cos();
    let cos_t = ((2.0 * cos_hoh + 1.0) / 3.0).sqrt();
    (cos_t, (1.0 - cos_t * cos_t).sqrt())
}

fn h3o_cut() -> StretchCut<4> {
    let (cos_t, sin_t) = cone();
    let frozen = |k: usize| {
        let phi = 2.0 * std::f64::consts::PI * (k as f64) / 3.0;
        [
            R_OH_H3O * sin_t * phi.cos(),
            R_OH_H3O * sin_t * phi.sin(),
            R_OH_H3O * cos_t,
        ]
    };
    let phi2 = 4.0 * std::f64::consts::PI / 3.0;
    StretchCut::new(
        [OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN],
        [[0.0, 0.0, 0.0], frozen(0), frozen(1), [0.0; 3]],
        3,
        [0.0, 0.0, 0.0],
        [sin_t * phi2.cos(), sin_t * phi2.sin(), cos_t],
        "r(O-H3), bohr: one O-H bond of the staked C3v pyramid, the other two frozen",
    )
}

const CHANNEL_A: [Fragment; 2] = [
    Fragment { slots: 0b0111, charge: 0, label: "H2O" },
    Fragment { slots: 0b1000, charge: 1, label: "H+" },
];
const CHANNEL_B: [Fragment; 2] = [
    Fragment { slots: 0b0111, charge: 1, label: "H2O+" },
    Fragment { slots: 0b1000, charge: 0, label: "H" },
];

fn manifest_row(name: &str, file: &str, t: &IonTable<4>, interp: f64, points: usize) -> String {
    let m = &t.meta;
    let (na, nb) = m.key.partition();
    let well = match m.well {
        Some(w) => format!(
            "{{\"q_e\": {:?}, \"D_e\": {:?}, \"k_e\": {:?}}}",
            w.r_e, w.d_e, w.k_e
        ),
        None => "null".to_string(),
    };
    format!(
        "    {{\"species\": \"{name}\", \"file\": \"{file}\", \"slots\": [{}], \
         \"class_z\": [{}], \"charge\": {}, \"sz2\": {}, \"n_electrons\": {}, \
         \"n_alpha\": {na}, \"n_beta\": {nb}, \"coordinate\": \"{}\", \
         \"q_min_bohr\": {:?}, \"q_max_bohr\": {:?}, \"n_grid\": {}, \
         \"n_basis\": {}, \"n_determinants\": {}, \"solver_route\": \"{}\", \
         \"exact_in_model\": {}, \"solver_exit\": \"{}\", \"uncertainty_hartree\": {:?}, \
         \"solver_budget_iterations\": {}, \"interpolant_uncertainty_hartree\": {:?}, \
         \"interpolant_points\": {points}, \"device_class\": \"{}\", \
         \"E_asymptote\": {:?}, \"asymptote_channel\": \"{}\", \
         \"boundary_systematic_hartree\": {:?}, \"well\": {well}}}",
        m.symbols
            .iter()
            .map(|s| format!("\"{s}\""))
            .collect::<Vec<_>>()
            .join(", "),
        m.key
            .class()
            .zs()
            .iter()
            .map(|z| z.to_string())
            .collect::<Vec<_>>()
            .join(", "),
        m.key.charge(),
        m.key.sz2(),
        m.key.n_electrons(),
        m.coordinate,
        m.q_min,
        m.q_max,
        t.q.len(),
        m.n_basis,
        m.n_det,
        match m.route {
            holon_chem::fci::SolverRoute::Determinant => "determinant",
            holon_chem::fci::SolverRoute::Dmrg => "DMRG",
        },
        m.route.is_exact_in_model(),
        m.exit.label(),
        m.worst_residual,
        m.solver_budget_iterations,
        interp,
        m.device,
        m.e_asymptote,
        m.asymptote_channel,
        m.boundary_systematic,
    )
}

/// One refused entry, rendered so a consumer of the bank learns what it may not have and
/// what would lift the refusal.
fn refusal_row(species: &str, why: &TableRefusal) -> String {
    let (kind, detail) = match why {
        TableRefusal::AnionFenced { fence, charge, cause } => (
            "AnionFenced",
            format!("\"fence\": \"{fence}\", \"charge\": {charge}, \"cause\": \"{cause}\""),
        ),
        TableRefusal::PastAutomaticRoute { n_det, n_orb, det_price_bytes, mpo_price_bytes } => (
            "PastAutomaticRoute",
            format!(
                "\"n_determinants\": {n_det}, \"n_orbitals\": {n_orb}, \
                 \"determinant_working_set_bytes\": {det_price_bytes}, \
                 \"mpo_price_bytes_provisional\": {mpo_price_bytes}, \
                 \"note\": \"Refused by THIS machine's resource door at these prices, not \
                 by any cap: a machine that admits the reservation runs it. Exit: more RAM, \
                 GANTT node F (GPU solve), or the DMRG cluster seam.\""
            ),
        ),
        other => ("Other", format!("\"debug\": \"{other:?}\"")),
    };
    format!("    {{\"species\": \"{species}\", \"refusal\": \"{kind}\", {detail}}}")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "docs/atoms/tables/ions".to_string());
    std::fs::create_dir_all(&out).expect("output directory");

    // ---------------------------------------------------------------- what is SERVED
    let cut = h3o_cut();
    let channels = [
        Channel { label: "H2O + H+", fragments: &CHANNEL_A },
        Channel { label: "H2O+ + H", fragments: &CHANNEL_B },
    ];
    let t0 = Instant::now();
    let table = generate_ion_table(&cut, 1, Q_MIN, Q_MAX, KNOTS, &channels)
        .expect("the staked H3O+ table generates");
    let gen_s = t0.elapsed().as_secs_f64();
    println!(
        "H3O+ : {} knots, {} determinants, route {:?}, exit {:?}, worst residual {:.3e}, \
         budget {}, {gen_s:.1} s",
        table.q.len(),
        table.meta.n_det,
        table.meta.route,
        table.meta.exit,
        table.meta.worst_residual,
        table.meta.solver_budget_iterations
    );

    // The FULL held-out sweep: every interval of the grid, at the three staked offsets.
    let d = 1.0 / (2.0 * 3.0f64.sqrt());
    let positions = [0.5 - d, 0.5, 0.5 + d];
    let intervals: Vec<usize> = (0..KNOTS - 1).collect();
    let t1 = Instant::now();
    let (worst_e, worst_f, points) = interpolant_error(&table, &cut, &intervals, &positions);
    println!(
        "  interpolant: {points} held-out points over {} intervals, worst |dE| {worst_e:.4e} Ha, \
         worst |dF| {worst_f:.4e} Ha/bohr, {:.1} s",
        intervals.len(),
        t1.elapsed().as_secs_f64()
    );
    for c in &table.meta.channels {
        println!("  channel {:<10} sum {:.12} Ha", c.label, c.sum);
    }
    println!(
        "  asymptote {:.12} Ha from '{}'; boundary systematic {:.4e} Ha",
        table.meta.e_asymptote, table.meta.asymptote_channel, table.meta.boundary_systematic
    );

    let published = table.with_interpolant_uncertainty(worst_e);
    let path = format!("{out}/H3O_plus.json");
    std::fs::write(&path, published.to_json().expect("a measured table serialises")).expect("write");
    println!("  -> {path}");

    // ---------------------------------------------------------------- what is REFUSED
    let oh_cut = StretchCut::new(
        [OXYGEN, HYDROGEN],
        [[0.0; 3]; 2],
        1,
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        "r(O-H), bohr",
    );
    let anion_frags = [
        Fragment { slots: 0b01, charge: -1, label: "O-" },
        Fragment { slots: 0b10, charge: 0, label: "H" },
    ];
    let oh_minus = generate_ion_table(
        &oh_cut,
        -1,
        1.0,
        12.0,
        KNOTS,
        &[Channel { label: "O- + H", fragments: &anion_frags }],
    )
    .expect_err("OH- must be REFUSED under fence I-5, not generated");

    let dimer_species = [OXYGEN, OXYGEN, HYDROGEN, HYDROGEN, HYDROGEN, HYDROGEN, HYDROGEN];
    let dimer_cut = StretchCut::new(
        dimer_species,
        [[0.0; 3]; 7],
        6,
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        "O-O separation, bohr (never reached: priced out)",
    );
    let dimer_frags = [
        Fragment { slots: 0b0011111, charge: 1, label: "H3O+" },
        Fragment { slots: 0b1100000, charge: 0, label: "H2O" },
    ];
    let dimer = generate_ion_table(
        &dimer_cut,
        1,
        2.0,
        12.0,
        KNOTS,
        &[Channel { label: "H3O+ + H2O", fragments: &dimer_frags }],
    )
    .expect_err("(H3O+ . H2O) must be priced out before it is attempted");
    println!("  REFUSED OH-          : {oh_minus:?}");
    println!("  REFUSED (H3O+ . H2O) : {dimer:?}");

    // ---------------------------------------------------------------------- manifest
    let mut manifest = String::from("{\n  \"schema\": \"ION-TABLES/bank/v1\",\n");
    manifest.push_str(
        "  \"note\": \"Charged curves along staked one-dimensional cuts, produced through \
         the SAME generic door the neutral curves go through (holon_chem::ion_table). \
         EVERY row names its charge AND its spin sector: a row that does not is unusable, \
         and the key type makes one impossible to construct. Two uncertainties are carried \
         and never conflated -- uncertainty_hartree is the worst Davidson residual (the \
         SOLVE, meaningless without solver_budget_iterations beside it) and \
         interpolant_uncertainty_hartree is the worst held-out departure of the table's \
         cubic Hermite interpolant from a direct solve (the GRID, which is what a consumer \
         integrating the curve is exposed to). Nothing is relaxed: fragments are frozen at \
         their in-complex geometry, so every D_e OVERSTATES the relaxed depth. \
         EXACT-IN-MODEL for STO-3G FCI, never compared to experiment.\",\n",
    );
    manifest.push_str(
        "  \"prereg\": \"conformance/atomworld/ION_TABLES_PREREG.md\",\n  \
         \"register\": \"conformance/water_observatory/ION_STAKING.md row I-2\",\n",
    );
    manifest.push_str("  \"tables\": [\n");
    manifest.push_str(&manifest_row("H3O+", "H3O_plus.json", &published, worst_e, points));
    manifest.push_str("\n  ],\n");
    manifest.push_str(
        "  \"refused_note\": \"A bank that silently omits what it will not serve teaches a \
         consumer nothing: an absent file looks like a file nobody made. Every refusal \
         below names its cause and its exit.\",\n",
    );
    manifest.push_str("  \"refused\": [\n");
    manifest.push_str(&refusal_row("OH-", &oh_minus));
    manifest.push_str(",\n");
    manifest.push_str(&refusal_row("H3O+ . H2O", &dimer));
    manifest.push_str("\n  ]\n}\n");
    let mpath = format!("{out}/manifest.json");
    std::fs::write(&mpath, manifest).expect("write manifest");
    println!("manifest -> {mpath}");
}
