// Measure the SHIPPED wasm artifact: how long the browser takes to solve the curve, and
// how far its answer is from the pinned 50-digit referee.
//
// This is not part of `cargo test`, and that is the gap it closes rather than a
// shortcoming. `holon-chem`'s referee gate runs against the NATIVE build, whose libm is
// the host's; the browser runs Rust's own libm compiled to wasm, which is a different
// implementation of `exp`, `powf` and `sqrt`. The native residual is therefore an
// inference about the browser, not a measurement of it -- and the two do differ, at
// 1e-15. This measures the artifact that actually ships.
//
//   ./build-web.sh
//   node check-wasm.mjs . viewer/holon_render.wasm
//
// Needs node and a curve emitted by the native path for the knot-by-knot comparison:
//   cargo run -p holon-chem --release --example emit_curve -- native_curve.json 492
//
// Optional: nothing in CI depends on it, because CI has no node. Run it when the
// generator, the special functions or the toolchain change.

import { readFileSync } from "node:fs";
const SP = process.argv[2];
const bytes = readFileSync(process.argv[3]);
const instance = new WebAssembly.Instance(new WebAssembly.Module(bytes), {});
const w = instance.exports;
const native = JSON.parse(readFileSync(`${SP}/native_curve.json`, "utf8"));

// Warm: same reason the native example warms -- the first call pays JIT tiering that
// the steady-state number should not be blamed for. Both are reported.
const t_cold0 = performance.now();
let st = w.holon_table_generate(0.3, 10.0, 492);
const cold = performance.now() - t_cold0;
if (st !== 1) throw new Error(`generate returned ${st}`);

const runs = [];
for (let k = 0; k < 9; k++) {
  const t0 = performance.now();
  st = w.holon_table_generate(0.3, 10.0, 492);
  runs.push(performance.now() - t0);
  if (st !== 1) throw new Error(`generate returned ${st}`);
}
runs.sort((a, b) => a - b);
console.log(`wasm/V8 generate(0.3, 10, 492): cold ${cold.toFixed(1)} ms, ` +
  `warm median ${runs[4].toFixed(1)} ms, min ${runs[0].toFixed(1)} ms, max ${runs[8].toFixed(1)} ms`);
console.log(`  knots ${w.holon_table_knots()}, curvature column ${w.holon_table_has_curvature()}, ` +
  `residual ${w.holon_table_residual().toExponential(3)} vs alt ${w.holon_table_residual_alt().toExponential(3)}`);

const cmp = (name, got, want) => {
  const d = Math.abs(got - want);
  console.log(`  ${name.padEnd(12)} wasm ${got.toPrecision(17)}  native ${want.toPrecision(17)}  |d| ${d.toExponential(3)}`);
  return d;
};
let worstScalar = 0;
worstScalar = Math.max(worstScalar, cmp("R_e", w.holon_table_r_e(), native.R_e));
worstScalar = Math.max(worstScalar, cmp("D_e", w.holon_table_d_e(), native.D_e));
worstScalar = Math.max(worstScalar, cmp("E_asymptote", w.holon_table_asymptote(), native.E_asymptote));

// Every knot: the table stores U = E - E_asymptote, and the Hermite interpolant
// reproduces the knot value exactly at a knot, so holon_curve_u at the native R grid
// reads the wasm's own knots back.
let worst = 0, at = 0;
for (let i = 0; i < native.R_grid_bohr.length; i++) {
  const R = native.R_grid_bohr[i];
  const u_native = native.E_hartree[i] - native.E_asymptote;
  const d = Math.abs(w.holon_curve_u(R) - u_native);
  if (d > worst) { worst = d; at = R; }
}
console.log(`  wasm vs native, all ${native.R_grid_bohr.length} knots: max |dU| = ${worst.toExponential(3)} Eh at R = ${at}`);
console.log(`  banner: residual ${w.holon_chem_referee_residual().toExponential(1)} Eh, ` +
  `referee 0x${(w.holon_chem_referee_digest() >>> 0).toString(16)}, ${w.holon_chem_referee_points()} points`);

// The residual the banner quotes was measured on the NATIVE build. The browser runs a
// different libm (Rust's own, compiled to wasm) and has already been shown to disagree
// with glibc at the 1e-15 level, so the native number is an inference about the browser,
// not a measurement of it. This measures it: the wasm's own knots against the pinned
// 50-digit referee, at the referee's own separations.
const ref = JSON.parse(readFileSync(
  new URL("../holon-chem/tests/data/referee_h2_sto3g_fci.json", import.meta.url), "utf8"));
const asym = w.holon_table_asymptote();
let wr = 0, wrAt = 0, wi = 0, wiAt = 0, wf = 0;
for (let i = 0; i < ref.R_grid_bohr.length; i++) {
  const R = parseFloat(ref.R_grid_bohr[i]);
  const E = parseFloat(ref.E_hartree[i]);
  const d = Math.abs(w.holon_chem_energy(R) - E);
  if (d > wr) { wr = d; wrAt = R; }
  const di = Math.abs((w.holon_curve_u(R) + asym) - E);
  if (di > wi) { wi = di; wiAt = R; }
  const df = Math.abs(w.holon_chem_force(R) - parseFloat(ref.F_hartree_per_bohr[i]));
  if (df > wf) { wf = df; }
}
console.log(`  WASM MODEL vs the 50-digit referee, all ${ref.R_grid_bohr.length} separations: ` +
  `max |dE| = ${wr.toExponential(3)} Eh at R = ${wrAt}, max |dF| = ${wf.toExponential(3)} Eh/a0`);
console.log(`  WASM TABLE (interpolant) vs the same: max |dE| = ${wi.toExponential(3)} Eh at R = ${wiAt}`);
console.log(`  (staked bound 1e-12; banner constant ${w.holon_chem_referee_residual().toExponential(1)})`);
