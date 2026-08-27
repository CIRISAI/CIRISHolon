// Wasm half of the portability check: instantiate the probe module and print the same
// lines `examples/native_probe.rs` prints, as raw f64 bit patterns and u64 digests.
//
//   node tools/portability_check.mjs <module.wasm>
//
// wasm32-unknown-unknown modules here import nothing, so no import object is needed.
// Any import a build does acquire is stubbed rather than serviced, because the probe
// never calls one — a stub that fires is a bug report, not a fallback.
import { readFile } from 'node:fs/promises';

const wasmPath = process.argv[2];
const bytes = await readFile(wasmPath);
const mod = await WebAssembly.compile(bytes);

const imports = {};
for (const imp of WebAssembly.Module.imports(mod)) {
  imports[imp.module] ??= {};
  imports[imp.module][imp.name] =
    imp.kind === 'function'
      ? () => { throw new Error(`probe called host import ${imp.module}.${imp.name}`); }
      : imp.kind === 'memory'
      ? new WebAssembly.Memory({ initial: 17 })
      : 0;
}
const { exports: e } = await WebAssembly.instantiate(mod, imports);

// f64 -> the same 16 hex digits Rust's `f64::to_bits` would print.
const view = new DataView(new ArrayBuffer(8));
const bits = (x) => {
  view.setFloat64(0, x);
  return view.getUint32(0).toString(16).padStart(8, '0') +
         view.getUint32(4).toString(16).padStart(8, '0');
};
// i64 crosses the boundary as a BigInt; render it unsigned.
const u64 = (b) => BigInt.asUintN(64, b).toString(16).padStart(16, '0');

for (let s = 0; s < 5; s++) {
  const n = e.probe_scenario_len(s);
  for (let i = 0; i < n; i++) console.log(`${s} ${i} ${bits(e.probe_scenario_value(s, i))}`);
}
[0.0, 0.5, 1.0, 2.0, 100.0].forEach((t, k) =>
  console.log(`coarsen ${k} ${e.probe_coarsen_classes(t)}`));

for (let n = 0; n < 2; n++)
  for (let w = 0; w < 9; w++) console.log(`chk ${n} ${w} ${u64(e.probe_field_digest(n, w))}`);

for (let n = 0; n < 3; n++) {
  console.log(`eig ${n} 0 ${u64(e.probe_eigensolve_digest(n, 0))}`);
  console.log(`eig ${n} 1 ${u64(e.probe_eigensolve_digest(n, 1))}`);
  console.log(`sweeps ${n} ${e.probe_jacobi_sweeps(n)}`);
  console.log(`conv ${n} ${e.probe_jacobi_converged(n)}`);
}

// The knife edge: the exact double at which the iteration count changes.
const edge = BigInt.asUintN(64, e.probe_sweep_boundary_bits());
console.log(`edge bits ${u64(edge)}`);
for (const [tag, b] of [['below', edge - 1n], ['at', edge], ['above', edge + 1n]])
  console.log(`edge ${tag} ${e.probe_sweeps_at_bits(BigInt.asIntN(64, b))}`);
