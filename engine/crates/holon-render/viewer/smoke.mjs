// Smoke-test the browser artifact's pair-table bank, in the artifact itself.
//
//   cd engine && node crates/holon-render/viewer/smoke.mjs
//
// # Why this exists
//
// `tests/mixtures.rs` gates the bank as a Rust library. Nothing gated it as the thing the
// page actually loads: a `wasm32-unknown-unknown` build, driven through raw `extern "C"`
// scalars, with the shipped JSON tables parsed by a host. Every defect this file has
// caught lived in exactly that gap:
//
//   * `holon_bank_clear` left atoms carrying species the bank had just forgotten, so the
//     scene stopped dead. Correct behaviour and an unreadable one; `Sim::clear_bank` now
//     does both halves. This file found it by the scene not moving.
//   * the viewer's feature detection asked for `holon_atom_z` and `holon_set_atom_z`,
//     which the engine has never exported (the species reading is
//     `holon_atom_species_z`), so the palette could never spawn.
//
// It is a script rather than a `#[test]` because the thing under test IS the wasm, and a
// Rust test would link the native rlib and prove nothing about it.
//
// Nothing here asserts: every line prints what it got and what it expected, because the
// point is to be read after a change to the ABI or to the shipped tables.

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const tables = join(here, "../../../../docs/atoms/tables");
const { instance } = await WebAssembly.instantiate(
  readFileSync(join(here, "holon_render.wasm")), {});
const w = instance.exports;

console.log("=== 1. the bank, solved in-browser ===");


const need = ["holon_bank_generate_pair","holon_bank_slot","holon_bank_filled","holon_bank_route",
  "holon_bank_source","holon_bank_n_det","holon_bank_uncertainty","holon_bank_claimed_exact",
  "holon_bank_provenance_ok","holon_bank_refusal_reason","holon_trimer_h_only",
  "holon_d1_validated","holon_pairs_ready","holon_atom_species_z","holon_set_atom_species",
  "holon_bank_pair_route","holon_bank_pair_is_heavy","holon_bank_pair_n_basis",
  "holon_bank_table_begin","holon_bank_table_knot","holon_bank_table_finish","holon_bank_clear"];
const missing = need.filter((n) => typeof w[n] !== "function");
console.log("missing exports:", missing.length ? missing : "none");

console.log("H2 generate:", w.holon_table_generate(0.3, 10.0, 96), "(1 = Ok)");
w.holon_reset(4);
console.log("pairs_ready:", w.holon_pairs_ready(), " provenance_ok:", w.holon_bank_provenance_ok());
console.log("fence h3-only:", w.holon_trimer_h_only(), " D1 validated:", w.holon_d1_validated());
const s00 = w.holon_bank_slot(1,1);
console.log(`slot(H,H)=${s00} filled=${w.holon_bank_filled(s00)} route=${w.holon_bank_route(s00)} `
  + `source=${w.holon_bank_source(s00)} n_det=${w.holon_bank_n_det(s00)} `
  + `unc=${w.holon_bank_uncertainty(s00)} R_e=${w.holon_bank_r_e(s00).toFixed(6)} D_e=${w.holon_bank_d_e(s00).toFixed(6)}`);

// Make atom 3 helium: a light pair, so the browser solves it here.
console.log("set atom3 -> He:", w.holon_set_atom_species(3, 2));
console.log("after species change, pairs_ready:", w.holon_pairs_ready(), "(0 expected: H-He and He-He are unbanked)");
console.log("H-He route:", w.holon_bank_pair_route(1,2), "heavy:", w.holon_bank_pair_is_heavy(1,2),
            "n_basis:", w.holon_bank_pair_n_basis(1,2));
let t = Date.now();
console.log("generate H-He:", w.holon_bank_generate_pair(1,2,96), `(${Date.now()-t} ms)`);
console.log("pairs_ready now:", w.holon_pairs_ready(), "(He-He not needed: only one He atom)");
const s01 = w.holon_bank_slot(1,2);
console.log(`slot(H,He)=${s01} filled=${w.holon_bank_filled(s01)} D_e=${w.holon_bank_d_e(s01)} `
  + `(0 = does not bind, which is the in-model truth)`);

// The heavy side of the split: Cl2 must be REFUSED for in-browser solving.
console.log("Cl2 heavy?", w.holon_bank_pair_is_heavy(17,17), "n_basis:", w.holon_bank_pair_n_basis(17,17),
            "n_det:", w.holon_bank_pair_n_det(17,17));
w.holon_bank_clear();
w.holon_reset(2);
w.holon_set_atom_species(0, 17); w.holon_set_atom_species(1, 17);
t = Date.now();
console.log("generate Cl2 in browser:", w.holon_bank_generate_pair(17,17,96),
            `(${Date.now()-t} ms; 21 = SplitViolated expected, NOT a 100-second solve)`);

// Step the H2 scene and check the ledger.
w.holon_bank_clear();
w.holon_table_generate(0.3, 10.0, 96);
w.holon_reset(2);
for (let i = 0; i < 200; i += 1) w.holon_step_frame(64);
console.log(`after 200x64: drift=${w.holon_drift_peak().toExponential(3)} bound=${w.holon_drift_bound().toExponential(3)} `
  + `energy_gate=${w.holon_energy_gate()} momentum_gate=${w.holon_momentum_gate()}`);

console.log("\n=== 2. the shipped-table door, and the provenance gate on it ===");


function loadShipped(file, za, zb, mutate) {
  const f = JSON.parse(readFileSync(join(tables, file), "utf8"));
  if (mutate) mutate(f);
  const slot = w.holon_bank_slot(za, zb);
  if (slot < 0) return `no slot for (${za},${zb})`;
  const r = f.R_grid_bohr;
  if (!w.holon_bank_table_begin(slot, r.length)) return "grid refused";
  for (let i = 0; i < r.length; i += 1) {
    w.holon_bank_table_knot(slot, i, r[i], f.E_hartree[i], f.F_hartree_per_bohr[i]);
    w.holon_bank_table_knot_curvature(slot, i, f.E2_hartree_per_bohr2[i]);
  }
  const route = f.solver_route === "determinant" ? 1 : f.solver_route === "DMRG" ? 2 : 0;
  return w.holon_bank_table_finish(slot, f.bound ? f.R_e : 0, f.bound ? f.D_e : 0,
    f.E_asymptote, route, f.species.n_determinants, f.species.n_basis,
    f.uncertainty_hartree, f.exact_in_model ? 1 : 0);
}

// A real mixed scene: 2 H + 2 Cl, H-H solved here, H-Cl and Cl-Cl shipped.
w.holon_table_generate(0.3, 10.0, 96);
w.holon_reset(4);
w.holon_set_atom_species(2, 17);
w.holon_set_atom_species(3, 17);
console.log("pairs_ready before shipped load:", w.holon_pairs_ready(), "(0 expected)");
console.log("HCl  ->", loadShipped("HCl.json", 1, 17), "(1 = Ok)");
console.log("Cl2  ->", loadShipped("Cl2.json", 17, 17), "(1 = Ok)");
console.log("pairs_ready after:", w.holon_pairs_ready(), " provenance_ok:", w.holon_bank_provenance_ok());
for (const [za, zb, name] of [[1,1,"H-H"],[1,17,"H-Cl"],[17,17,"Cl-Cl"]]) {
  const s = w.holon_bank_slot(za, zb);
  console.log(`  ${name.padEnd(6)} slot=${s} route=${w.holon_bank_route(s)} source=${w.holon_bank_source(s)} `
    + `n_det=${w.holon_bank_n_det(s)} unc=${w.holon_bank_uncertainty(s).toExponential(2)} `
    + `knots=${w.holon_bank_knots(s)} R_e=${w.holon_bank_r_e(s).toFixed(4)} D_e=${w.holon_bank_d_e(s).toFixed(6)}`);
}
// Place a real HCl dimer and read the bond.
w.holon_set_position(0, 10.0, 12.0); w.holon_set_position(2, 10.0 + 2.5369, 12.0);
// atoms are 0=H 1=H 2=Cl 3=Cl, so (1,3) is another H-Cl pair, not a Cl2 one.
w.holon_set_position(1, 30.0, 12.0); w.holon_set_position(3, 30.0 + 4.0241, 12.0);
for (let i = 0; i < 4; i += 1) w.holon_set_velocity(i, 0, 0);
w.holon_rebase();
for (let k = 0; k < w.holon_pair_count(); k += 1) {
  const i = w.holon_pair_i(k), j = w.holon_pair_j(k);
  if (w.holon_pair_r(k) > 8) continue;
  console.log(`  pair(${i},${j}) Z=${w.holon_atom_species_z(i)},${w.holon_atom_species_z(j)} `
    + `r=${w.holon_pair_r(k).toFixed(4)} e_rel=${w.holon_pair_e_rel(k).toExponential(4)} `
    + `slot=${w.holon_pair_slot(k)} bonded=${w.holon_pair_bonded(k)}`);
}
for (let i = 0; i < 100; i += 1) w.holon_step_frame(64);
console.log(`mixed scene 100x64: drift=${w.holon_drift_peak().toExponential(3)} `
  + `bound=${w.holon_drift_bound().toExponential(3)} energy_gate=${w.holon_energy_gate()} `
  + `momentum_gate=${w.holon_momentum_gate()} dt=${w.holon_dt().toFixed(5)}`);

// THE PLANT, through the shipped-table door: relabel Cl2 as DMRG-and-exact.
w.holon_bank_clear();
w.holon_table_generate(0.3, 10.0, 96);
w.holon_reset(2);
w.holon_set_atom_species(0, 17); w.holon_set_atom_species(1, 17);
const code = loadShipped("Cl2.json", 17, 17, (f) => { f.solver_route = "DMRG"; });
console.log("Cl2 relabelled DMRG-but-still-claiming-exact ->", code, "(17 = DmrgClaimedExact)");
console.log("  slot filled after refusal:", w.holon_bank_filled(w.holon_bank_slot(17,17)), "(0 = evicted)");
const code2 = loadShipped("Cl2.json", 17, 17, (f) => { f.solver_route = "DMRG"; f.exact_in_model = false; });
console.log("Cl2 relabelled DMRG, honest, D1 not admitted ->", code2, "(18 = DmrgUnvalidated)");
const code3 = loadShipped("Cl2.json", 17, 17, (f) => { f.uncertainty_hartree = 0; });
console.log("Cl2 with its uncertainty removed ->", code3, "(19 = UncertaintyMissing)");

console.log("\n=== 3. every engine call each page makes resolves against this wasm ===");
//
// A page calling an export that does not exist fails at the moment a user touches the
// control that calls it, which is the worst time to find out. This does not catch a
// SEMANTIC mismatch — `holon_atom_z` exists and is the atom's z COORDINATE, and the
// unified viewer read it as the species for months — but it catches the whole class where
// the name is simply gone, which is what an ABI rename does.
{
  const have = new Set(Object.keys(w));
  for (const page of [
    "../../../../docs/atoms/app.js",
    "../../../../docs/unified/app.js",
    "app.js",
    "unified/app.js",
  ]) {
    const js = readFileSync(join(here, page), "utf8");
    const used = new Set([...js.matchAll(/\bw\.(holon_[a-z0-9_]+)/g)].map((m) => m[1]));
    const missing = [...used].filter((n) => !have.has(n));
    console.log(
      `  ${page.replace("../../../../", "").padEnd(20)} ${String(used.size).padStart(3)} engine calls, `
      + `missing: ${missing.length ? missing.join(" ") : "none"}`,
    );
  }
}
