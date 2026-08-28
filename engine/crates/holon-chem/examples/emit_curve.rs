//! Write the engine-computed curve out as the `h2_potential.json` contract, and time it.
//!
//! The viewer computes its curve at load and does not need this file. It exists for the
//! FALLBACK path (a host that cannot run the generator, or a deliberate A/B against a
//! different curve) and for the timing line the wasm budget is argued from.
//!
//! ```text
//! cargo run -p holon-chem --release --example emit_curve -- [out.json] [n]
//! ```

use holon_chem::table::generate_table;
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = args.get(1).cloned();
    let n: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(492);
    let (r_min, r_max) = (0.3f64, 10.0f64);

    // Warm once, then time. The first call pays for page faults and branch predictors
    // that the browser also pays once; reporting the cold number as the steady-state one
    // would overstate the cost by a factor that has nothing to do with the physics.
    let _ = generate_table(r_min, r_max, 8);

    let t0 = Instant::now();
    let table = generate_table(r_min, r_max, n).expect("curve");
    let build = t0.elapsed();

    let t1 = Instant::now();
    let json = table.to_json();
    let serialise = t1.elapsed();

    let m = table.meta;
    println!("H2/STO-3G/FCI, computed in-engine (f64)");
    println!("  grid          {n} knots, R in [{r_min}, {r_max}] bohr, uniform in R^-1/4");
    println!("  R_e           {:.15} bohr", m.r_e);
    println!("  D_e           {:.15} hartree", m.d_e);
    println!("  E_asymptote   {:.15} hartree", m.e_asymptote);
    println!("  E(R_e)        {:.15} hartree", m.e_at_r_e);
    println!("  E_H_atom      {:.15} hartree", m.e_h_atom);
    let (de, df) = table.hermite_error(7);
    println!("  interpolant   max |dE| {de:.3e} Eh, max |dF| {df:.3e} Eh/a0");
    println!(
        "  build         {:.3} ms ({:.1} us/knot)",
        build.as_secs_f64() * 1e3,
        build.as_secs_f64() * 1e6 / n as f64
    );
    println!("  serialise     {:.3} ms, {} bytes", serialise.as_secs_f64() * 1e3, json.len());

    if let Some(path) = out {
        std::fs::write(&path, &json).expect("write curve");
        println!("  wrote         {path}");
    }
}
