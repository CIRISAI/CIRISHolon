// The workbench's browser artifact, RUN rather than built.
//
// ci-gates.sh gate 1 compiles holon-render for wasm; pages.yml rebuilds this page's engine
// from the commit being deployed, so what ships is always what that commit's source
// builds. Neither of those RUNS anything, and the atom viewer's history says why that gap
// matters: a rebuilt viewer wasm once trapped with `memory access out of bounds` on the
// cheapest solve it does while `cargo test` was fully green against the identical source,
// and the unified viewer read `holon_atom_z` — an atom's Z COORDINATE in bohr — as its
// species for months, guarded by a second export that has never existed.
//
// This gate runs the COMMITTED artifact, which is what a bare checkout gets and what a
// developer opening the page locally is looking at. It can therefore drift from source
// between deploys without breaking the deployed site — and if it drifts in any way that
// matters (a missing export, a changed refusal code, a fence that stops firing) the checks
// below say so, which is the part worth gating. Byte identity is not: it would go red on
// lanes editing holon-chem who have never heard of this page, to buy a guarantee the
// deploy step already provides by construction.
//
// So this drives the actual artifact this page ships, through the raw extern "C" ABI, in
// the same order `app.js` drives it. What it checks, and why each is here:
//
//   1. THE TWO CONTRACTS BETWEEN THE PAGE AND ITS WORLD. Every export `app.js` declares
//      resolves in the artifact, every `holon_*` the page CALLS is on that declared list,
//      and every element id the page WRITES to exists in the markup. All three lists are
//      read out of the shipped files rather than duplicated here, so they cannot drift.
//      `put()` is deliberately tolerant of a missing element so a removed panel cannot
//      take the frame loop down; the third check is what keeps that tolerance from
//      letting a renamed panel go dark in silence.
//   2. The pure-H preset boots as the page boots it, and the curve agrees with the pinned
//      50-digit referee to the figure the page displays.
//   3. Four hundred frames of real dynamics with the thermostat engaged leave BOTH
//      conservation gates closed and W_ext non-zero. The thermostat moves energy and
//      momentum, so this is the ledgered-thermostat claim, not a free-flight one.
//   4. The hand's work is a receipt: dragging posts exactly the spring energy into W_ext,
//      the energy gate holds through the drag, and release takes exactly the stored
//      energy back out.
//   5. The O:2H preset FENCES where the page says it fences — the (O,O) curve refused by
//      the engine's own in-browser split with the code the page NAMES, and O-bearing
//      triples refused AND counted, with a discriminating second arm proving that count
//      is not the uninformative "never looked" zero.
//   6. Readiness is asked after the composition, not before. Both readings are pinned,
//      because the page once reported a frozen box as "SETTLING · 0.0 K" on the strength
//      of asking too early.
//   7. The seeded scene replays bit-identically, with the three-body term live.
//   8. THE INVERTED CHECK. The page fences gravity, the barostat and the phase classifier
//      on the grounds that this engine has no such export. If any of them ever appears,
//      this gate FAILS — telling us to un-fence the panel. A fence justified by an absence
//      decays into a lie the day the absence ends, and nothing else here would notice.
//   9. No SYNTHETIC label survives into the rendered page and no synthesized telemetry
//      survives into the code.
//
// COST. Measured at about 30 s. It was 3m14s until the two dominant items were found and
// cut: the O-H solve is ~15 s of FIXED setup whatever the knot count (17.0 s at 4 knots
// against 14.7 s at 16), so the gate buys 8 rather than the page's 160 and skips it
// entirely in the arm whose subject is the fence counter; and the 14,157-node H3 surface
// was being regenerated five times, where two suffice. Neither cut removes an assertion.
//
// Exit 0 on success, 1 with a named failure otherwise. Node only; no dependencies.

import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const AU_TO_FS = 0.024188843265857;

// The repository root, FOUND rather than assumed. `join(here, "..", "..")` is right only
// while this file sits exactly two levels down, and it silently resolves to the wrong
// directory the moment the gate runs from a copy — which made every mutation test of the
// citation block fail with "artifact does not exist" instead of the defect it was probing.
// A check whose failures all look the same cannot tell you which one fired.
let repoRoot = here;
for (let up = 0; up < 6; up++) {
  try { readFileSync(join(repoRoot, ".git", "HEAD")); break; } catch { /* keep climbing */ }
  try { readFileSync(join(repoRoot, ".git")); break; } catch { /* worktrees: .git is a file */ }
  repoRoot = join(repoRoot, "..");
}

let failures = 0;
let passes = 0;

/// HOW MANY CHECKS RAN, reported on EVERY exit path.
///
/// This file threw once while being written — a botched `new Function` in the acuity
/// block — and node printed a stack trace and exited. The exit code was non-zero so CI
/// would have caught SOMETHING, but every check after the throw silently did not run and
/// the output named no failing property at all. A gate that can die halfway and report a
/// stack trace is a gate whose coverage is unknown exactly when you most need it.
///
/// A floor checked at the end of the script does not fix that: a throw never reaches the
/// end. An `exit` hook does, because it fires on the throw path too — so a crash now says
/// how far it got, right under the stack trace, instead of leaving that to be inferred.
///
/// The floor is deliberately well below the current total. Its job is to catch a run that
/// STOPPED EARLY, not to make adding a check a two-file edit; the number moves DOWN only
/// when checks are deliberately removed.
const MIN_CHECKS = 70;
let reachedEnd = false;
process.on("exit", () => {
  if (reachedEnd) return;
  console.log(`\n  FAIL the gate did not reach its end — only ${passes + failures} checks ran`);
  console.log("       coverage is UNKNOWN for everything after that point; fix the throw above");
});
function ok(what) { passes += 1; console.log(`  ok   ${what}`); }
function no(what, detail) {
  failures += 1;
  console.log(`  FAIL ${what}`);
  if (detail !== undefined) console.log(`       ${detail}`);
}
function want(cond, what, detail) { cond ? ok(what) : no(what, detail); }

// ---------------------------------------------------------------- 1. the export contract

const appSource = readFileSync(join(here, "app.js"), "utf8");
const listMatch = appSource.match(/const REQUIRED_EXPORTS = \[([\s\S]*?)\];/);
if (!listMatch) {
  console.log("  FAIL could not find REQUIRED_EXPORTS in app.js");
  process.exit(1);
}
const required = [...listMatch[1].matchAll(/"([a-z0-9_]+)"/g)].map((m) => m[1]);

// THE SECOND LIST, and the reason it is a second list rather than more of the first.
//
// `REQUIRED_EXPORTS` is a boot-time refusal: a wasm missing one of those is not this engine
// and the page says so with the name. `PENDING_EXPORTS` is the other kind of absence — the
// exports WB-10.1 and WB-10.2 are building right now, which the page must handle by FENCING
// the rows they serve, not by refusing to start. Putting them in the required list would
// take the whole page down for want of a nucleus readout, which is the opposite of a fence.
const pendingMatch = appSource.match(/const PENDING_EXPORTS = \[([\s\S]*?)\n\];/);
want(pendingMatch !== null, "the page's PENDING_EXPORTS list is where the gate expects it");
const pending = pendingMatch
  ? [...pendingMatch[1].matchAll(/name: "([a-z0-9_]+)"/g)].map((m) => m[1]) : [];
const declared = new Set([...required, ...pending]);

// The two lists must be DISJOINT. A name on both would be required at boot and fenced at
// render, and whichever ran first would decide what the page did — which is the shape of a
// guard that names a different function from the one it guards.
const onBoth = pending.filter((n) => required.includes(n));
want(onBoth.length === 0,
  `the required and pending export lists are disjoint (${required.length} + ${pending.length})`,
  onBoth.length ? `on both lists: ${onBoth.join(", ")} — a required export cannot also be `
    + "the reason a band fences, because the boot would already have refused" : undefined);

// Every `holon_*` the page actually CALLS, discovered from the source. The declared lists
// above are a promise; this is the check that the promise covers the calls. `?.(` forms are
// deliberately included: an optional call on an export that does not exist is the false
// guard the atom viewer shipped for months.
//
// STRING LITERALS COUNT AS CALLS, and that is not pedantry: the descent panel dispatches
// through `w[name](i)` over a table of names, so a typo'd export there would be invisible
// to a `w.holon_*` scan and would fence a row forever with nobody told. Any `"holon_*"` in
// the source is therefore required to be on one of the two lists.
const called = new Set([
  ...[...appSource.matchAll(/\bw\.(holon_[a-z0-9_]+)/g)].map((m) => m[1]),
  ...[...appSource.matchAll(/"(holon_[a-z0-9_]+)"/g)].map((m) => m[1]),
]);
const undeclared = [...called].filter((n) => !declared.has(n)).sort();

// The other half of the same contract: every element id the page WRITES to must exist in
// the markup. `put()` and `tag()` are deliberately tolerant of a missing element so that a
// removed panel cannot take the frame loop down, and that tolerance is exactly what would
// let a renamed panel go dark in silence. This is the check that makes the tolerance safe.
const htmlSource = readFileSync(join(here, "index.html"), "utf8");
const domIds = new Set([...htmlSource.matchAll(/\bid="([A-Za-z0-9_-]+)"/g)].map((m) => m[1]));
const written = new Set([
  ...appSource.matchAll(/\bput\("([A-Za-z0-9_-]+)"/g),
  ...appSource.matchAll(/\btag\("([A-Za-z0-9_-]+)"/g),
  ...appSource.matchAll(/\bdescField\("([A-Za-z0-9_-]+)"/g),
  ...appSource.matchAll(/\bUI\["([A-Za-z0-9_-]+)"\]/g),
].map((m) => m[1]));
const orphaned = [...written].filter((id) => !domIds.has(id)).sort();
want(orphaned.length === 0,
  `every element id app.js writes to exists in index.html (${written.size} of them)`,
  orphaned.length ? `written but absent from the markup: ${orphaned.join(", ")}` : undefined);

const bytes = readFileSync(join(here, "holon_render.wasm"));
const { instance } = await WebAssembly.instantiate(bytes, {});
const w = instance.exports;

const missing = required.filter((n) => typeof w[n] !== "function");
want(missing.length === 0, `every export app.js declares resolves (${required.length} of them)`,
  missing.length ? `missing: ${missing.join(", ")}` : undefined);
want(undeclared.length === 0, "every holon_* call in app.js is on the declared list",
  undeclared.length ? `called but undeclared: ${undeclared.join(", ")}` : undefined);

// THE PENDING LIST IS A COMMISSION, NOT A WISH. Every name on it must belong to a family
// FSD-W3 §11.4 actually commissions, so a typo or an invented export cannot sit there
// fencing a band indefinitely while looking like scheduled work. `holon_law_probe` is the
// one exception and it is declared as one IN THE PAGE: §11.4 commissions the PROPERTY
// (wasm == native to the bit, pinned by tests/wasm_law.rs) and names no export for it, so
// the page records the name's provenance as WB-10.3's brief rather than citing a line that
// does not carry it.
const fsdText = readFileSync(
  join(repoRoot, "conformance/water_observatory/WORKBENCH_FSD.md"), "utf8");
const COMMISSIONED = [
  [/^holon_nucleus_/, "holon_nucleus_*"],
  [/^holon_atom_band_/, "holon_atom_band_*"],
  [/^holon_atom_in_molecule$/, "holon_atom_in_molecule"],
];
for (const name of pending) {
  const fam = COMMISSIONED.find(([re]) => re.test(name));
  if (!fam) {
    // Not commissioned by §11.4: the page must say so itself, in the entry's own `spec`.
    const entry = pendingMatch[1].split(/\n  \{/).find((b) => b.includes(`"${name}"`)) || "";
    want(/spec: "[^"]*not named in §11\.4[^"]*"/.test(entry),
      `pending export ${name} is declared as NOT commissioned by §11.4`,
      "an export the FSD does not commission must say so in its own `spec` field; citing a "
      + "line that does not carry the claim is the failure this gate battery is about");
    continue;
  }
  want(fsdText.includes(fam[1]),
    `pending export ${name} belongs to a family §11.4 commissions (${fam[1]})`,
    `the FSD does not mention ${fam[1]}, so this name is not scheduled work — a band fenced `
    + "on an export nobody is building is a fence with no exit");
}
// AND THEY ALL RESOLVE TODAY, so that is what is demanded rather than reported.
//
// This started as an informational line while WB-10.1 and WB-10.2 were in build. They
// landed, the fine bands flipped live with no edit to the page — which is the property the
// export-gating was built for — and an informational line would now be the wrong instrument:
// a rebuild that dropped one of these would silently re-fence a live band and nothing would
// say so. The absence path is still covered, in the two places it belongs: `bandLiveness`
// and `exportRow` are lifted out and run against a stub artifact with the exports missing.
const pendingLive = pending.filter((n) => typeof w[n] === "function");
const pendingGone = pending.filter((n) => typeof w[n] !== "function");
want(pendingGone.length === 0,
  `every export the fine bands wait on resolves in the committed artifact (${pending.length})`,
  pendingGone.length
    ? `absent: ${pendingGone.join(", ")} — the atom and nucleus bands re-fence on this `
      + "artifact. If that is deliberate, say so here; if it is a rebuild that dropped a "
      + "symbol, the band went dark and only this line would have told you"
    : undefined);
ok(`${pendingLive.length} pending-list exports are live in this artifact, so the fine bands `
  + "flip without an edit to the page");

// ---------------------------------------------------------------- 2. the pure-H boot

w.holon_set_dims(1);
w.holon_set_boundary(0);
w.holon_set_census_enabled(1);
want(w.holon_dims() === 3, "the scene is three-dimensional", `holon_dims -> ${w.holon_dims()}`);

w.holon_bank_clear();
w.holon_bank_register(1);
const pairStatus = w.holon_table_generate(0.6, 12.0, 192);
want(pairStatus === 1, "the H-H curve is solved in the browser and banked",
  `holon_table_generate -> ${pairStatus}`);
want(w.holon_table_knots() === 192, "the curve carries the knots that were asked for",
  `${w.holon_table_knots()} knots`);

// The referee residual is a build constant, and holon-chem's tests/referee.rs enforces it
// on every build. Reproduced here because the PAGE displays it, and a number the page
// shows should be a number this gate has seen.
const residual = w.holon_chem_referee_residual();
want(residual > 0 && residual < 1e-12,
  `the curve agrees with the 50-digit referee (${residual.toExponential(3)} Ha over ${w.holon_chem_referee_points()} separations)`,
  `residual ${residual}`);

// The unit constant the page converts every rate with, taken from the artifact rather than
// trusted. `holon_period_fs` and `holon_period` are the same period in two units, so their
// ratio IS the engine's own AU_TO_FS. It is read AFTER the curve loads: the period is
// derived from the curve, so before one exists this is 0/0 — which is how the page's own
// version of this check was found throwing at boot.
const ratio = w.holon_period_fs() / w.holon_period();
want(Math.abs(ratio - AU_TO_FS) < 1e-15, "the engine's time unit matches the page's constant",
  `engine ${ratio}, page ${AU_TO_FS}`);

const trimerStatus = w.holon_trimer_generate();
want(trimerStatus === 1 && w.holon_trimer_loaded() === 1,
  `the H3 three-body surface generates in the browser (${w.holon_trimer_nodes()} nodes)`,
  `holon_trimer_generate -> ${trimerStatus}`);

// Calibration, timed on this side exactly as the page times it.
let sub = 2000, total = 0, elapsed = 0;
w.holon_calibration_burst(500);
const cal0 = performance.now();
while (performance.now() - cal0 < 200) {
  const a = performance.now();
  w.holon_calibration_burst(sub);
  const b = performance.now();
  elapsed += b - a; total += sub;
  if (b - a < 20) sub *= 2;
}
w.holon_set_calibration((total / elapsed) * 1000);
want(w.holon_calibrated() === 1 && w.holon_substeps_per_second() > 0,
  `the device calibrates (${w.holon_substeps_per_second().toExponential(2)} substeps/s here)`);

// ---------------------------------------------------------------- 3. gates close

w.holon_reset(12);
want(w.holon_atom_count() > 0 && w.holon_pairs_ready() === 1,
  `the pure-H scene opens and is ready to step (${w.holon_atom_count()} atoms)`);

// The opener's own promise: no scene opens handing out bonds nobody paid for.
want(w.holon_bonded_count() === 0, "the opener hands out no bonds",
  `${w.holon_bonded_count()} bonded pairs at t = 0`);

w.holon_set_thermostat(1, 293.15);
for (let f = 0; f < 400; f++) w.holon_step_frame(64);

want(w.holon_energy_gate() === 1,
  `the energy gate closes under a thermostatted run (drift ${w.holon_drift().toExponential(2)} vs bound ${w.holon_drift_bound().toExponential(2)} Ha)`,
  `drift ${w.holon_drift()} bound ${w.holon_drift_bound()}`);
want(w.holon_momentum_gate() === 1,
  `the momentum gate closes (residual ${w.holon_momentum_residual().toExponential(2)} vs bound ${w.holon_momentum_bound().toExponential(2)})`);
want(Math.abs(w.holon_w_ext()) > 0,
  `the thermostat's heat is posted to the ledger rather than excused (W_ext ${w.holon_w_ext().toExponential(3)} Ha)`,
  "W_ext is exactly zero after 400 thermostatted frames, which would mean the heat went nowhere");
want(w.holon_temperature() > 0 && w.holon_time() > 0,
  `time advances and the scene has a temperature (${w.holon_time() * AU_TO_FS} fs, ${w.holon_temperature().toFixed(1)} K)`);

// ---------------------------------------------------------------- 4. the ledgered hand

const wBefore = w.holon_w_ext();
w.holon_grab(0);
w.holon_move_anchor_3d(w.holon_atom_x(0) + 1.5, w.holon_atom_y(0), w.holon_atom_z(0));
const posted = w.holon_w_ext() - wBefore;
const spring = w.holon_e_spring();
want(Math.abs(posted - spring) < 1e-12,
  `the hand's work is posted exactly (dU ${posted.toExponential(4)} Ha == spring energy ${spring.toExponential(4)} Ha)`,
  `posted ${posted}, spring ${spring}`);
for (let f = 0; f < 50; f++) w.holon_step_frame(64);
want(w.holon_energy_gate() === 1, "the energy gate stays closed while the hand holds an atom");

// The claim about release is a DELTA, not an arithmetic identity against the value before
// the grab: the thermostat has been posting to the same column throughout, and the spring
// energy at release is not the one posted at the drag because the atom moved. Both terms
// are therefore sampled immediately before the call. Writing the identity the other way
// round is how this check first failed against a correct engine.
const springAtRelease = w.holon_e_spring();
const wBeforeRelease = w.holon_w_ext();
w.holon_release();
const removed = wBeforeRelease - w.holon_w_ext();
want(Math.abs(removed - springAtRelease) < 1e-12 && w.holon_energy_gate() === 1,
  `release takes exactly the stored spring energy back out of the receipt (${removed.toExponential(4)} Ha)`,
  `removed ${removed}, stored ${springAtRelease}, gate ${w.holon_energy_gate()}`);

// ---------------------------------------------------------------- 5. the fences fire

// ON A FRESH INSTANCE, and that is the whole point of this block.
//
// `holon_bank_clear` clears the pair bank and NOT the three-body surface, so an H3 surface
// generated earlier in this file survives into any later scene. A first attempt at this
// check ran on the instance above and passed while the thing it was meant to detect was
// planted — the surface it needed was already resident from the pure-H section. A check
// that cannot fail has not checked anything, so the fence arms below each get a clean
// engine and the two arms are run as a DISCRIMINATION rather than a single threshold.
async function freshEngine() {
  return (await WebAssembly.instantiate(bytes, {})).instance.exports;
}

// THE SHIPPED ARTIFACTS, read out of the page's own `SHIPPED` table so this gate pins what
// the page serves rather than a parallel list. Every pin is checked against the bytes in
// the tree (a re-emitted artifact under a stale pin is a fence on the page, and this is
// where it is caught first), and the water table in the served tree must be the committed
// one from holon-chem's test data byte for byte — a copy is not a second source.
const shippedMatch = appSource.match(/const SHIPPED = \{([\s\S]*?)\n\};/);
want(shippedMatch !== null, "the page's SHIPPED table is where the gate expects it");
const shippedEntries = shippedMatch
  ? [...shippedMatch[1].matchAll(/file: "([^"]+)",[\s\S]*?sha256: "([0-9a-f]{64})"/g)]
    .map((m) => ({ file: m[1], sha256: m[2] }))
  : [];
want(shippedEntries.length >= 2,
  `SHIPPED names at least the (H,O) curve and the (O,H,H) table (${shippedEntries.length} entries)`);
const sha256Of = (buf) => createHash("sha256").update(buf).digest("hex");
for (const s of shippedEntries) {
  let buf = null;
  try { buf = readFileSync(join(here, s.file)); } catch { /* below */ }
  want(buf !== null, `shipped artifact ${s.file} is in the served tree`);
  if (buf) {
    const got = sha256Of(buf);
    want(got === s.sha256, `${s.file} digests to its pin (${s.sha256.slice(0, 12)}…)`,
      `the tree has ${got.slice(0, 12)}…; the page would fence this artifact rather than serve it`);
  }
}
const waterCanonical = readFileSync(join(repoRoot, "engine/crates/holon-chem/tests/data/s2/s2_water_table.txt"));
want(sha256Of(waterCanonical) === sha256Of(readFileSync(join(here, "tables/s2_water_table.txt"))),
  "the served (O,H,H) table is the committed one byte for byte");
// the emitter's file names: "HO" for a heteronuclear pair in Z order, "O2" for a homonuclear one
const shippedPairJson = (za, zb) => {
  const [lo, hi] = za <= zb ? [za, zb] : [zb, za];
  const name = lo === hi ? `${sym2(lo)}2` : `${sym2(lo)}${sym2(hi)}`;
  const entry = shippedEntries.find((s) => s.file.endsWith(`/${name}.json`));
  return entry ? JSON.parse(readFileSync(join(here, entry.file), "utf8")) : null;
};
function sym2(z) { return z === 1 ? "H" : z === 8 ? "O" : `Z${z}`; }

/// The page's push, mirrored: the bank's node-wise door, finish code returned.
function pushShippedPair(e, za, zb, j) {
  const slot = e.holon_bank_slot(za, zb);
  if (slot < 0) return -1;
  const n = j.R_grid_bohr.length;
  if (e.holon_bank_table_begin(slot, n) !== 1) return -2;
  for (let i = 0; i < n; i++) {
    if (e.holon_bank_table_knot(slot, i, j.R_grid_bohr[i], j.E_hartree[i], j.F_hartree_per_bohr[i]) !== 1) return -3;
    if (e.holon_bank_table_knot_curvature(slot, i, j.E2_hartree_per_bohr2[i]) !== 1) return -4;
  }
  return e.holon_bank_table_finish(slot, j.R_e, j.D_e, j.E_asymptote,
    j.solver_route === "determinant" ? 1 : 2, j.species.n_determinants, j.species.n_basis,
    j.uncertainty_hartree, j.exact_in_model ? 1 : 0);
}

/// The page's water push, mirrored: bytes into the reservation, then the loader.
function pushWater(e, buf) {
  const ptr = e.holon_water_table_alloc(buf.length);
  new Uint8Array(e.memory.buffer, ptr, buf.length).set(buf);
  return e.holon_water_table_load();
}

/// Build the O:2H scene exactly as `loadPreset` builds it, optionally WITHOUT the H3
/// surface, and report what the engine did.
///
/// The O-H curve arrives the way the page gets it: the shipped `tables/HO.json` pushed
/// through the bank's door. It is on the heavy side of the split since the split was
/// re-measured on this engine (the solve is a fixed ~5 s of setup before any knot, over
/// the page's 5 s budget), and the split refuses a shipped file for a pair it expects the
/// browser to solve — so "served" here means the bank's provenance gate ADMITTED the file,
/// which is a verdict of the engine's, not of this script. The blind arm skips the push
/// because its subject is the fence counter, which does not depend on it.
async function o2hScene({ withTrimer, solveOH = true }) {
  const e = await freshEngine();
  e.holon_set_dims(1);
  e.holon_set_boundary(0);
  e.holon_set_census_enabled(1);
  e.holon_bank_clear();
  e.holon_bank_register(1);
  e.holon_bank_register(8);
  e.holon_table_generate(0.6, 12.0, 192);
  const hoJson = shippedPairJson(8, 1);
  const oh = solveOH && hoJson ? pushShippedPair(e, 8, 1, hoJson) : null;
  const oo = e.holon_bank_generate_pair(8, 8, 160);
  if (withTrimer) e.holon_trimer_generate();
  e.holon_reset(12);
  for (let i = 0; i < e.holon_atom_count(); i++) e.holon_set_atom_species(i, i % 3 === 0 ? 8 : 1);
  e.holon_reset(12);
  for (let i = 0; i < e.holon_atom_count(); i++) e.holon_set_atom_species(i, i % 3 === 0 ? 8 : 1);
  e.holon_rebase();
  for (let f = 0; f < 40; f++) e.holon_step_frame(64);
  let oxygen = 0;
  for (let i = 0; i < e.holon_atom_count(); i++) if (e.holon_atom_species_z(i) === 8) oxygen += 1;
  return { e, oh, oo, oxygen, fenced: e.holon_fence_untabulated() };
}

const served = await o2hScene({ withTrimer: true });
want(served.oh === 1, "the shipped O-H curve is ADMITTED by the bank's provenance gate (finish code 1)",
  `holon_bank_table_finish for (8,1) -> ${served.oh}; -1..-4 are this script's own push refusals, `
  + "21 is the split refusing a shipped file for a pair it expects the browser to solve");
// The other side of the same split, on the same engine: the in-browser solve of that pair
// is refused BEFORE it runs. The slot is already filled by the shipped file above and the
// refusal must not touch it.
const ohInBrowser = served.e.holon_bank_generate_pair(8, 1, served.e.holon_bank_browser_knots());
want(ohInBrowser === 21,
  `the in-browser O-H solve is refused by the re-measured split (code 21, predicted `
  + `${served.e.holon_bank_pair_predicted_seconds(8, 1).toFixed(0)} s against `
  + `${served.e.holon_bank_browser_budget_seconds().toFixed(0)} s): it is a shipped curve now`,
  `expected 21, got ${ohInBrowser}`);
want(served.e.holon_bank_filled_count() === 2,
  "both curves the O:2H scene can serve are in the bank (H-H solved here, H-O shipped)",
  `filled ${served.e.holon_bank_filled_count()}`);

// 21 == PROVENANCE_REFUSED (16) + Refusal::SplitViolated (5). The page names this code in
// the text it shows the user, so the gate pins it: if the engine renumbers its refusals,
// the page's fence would go on explaining the wrong one and nothing else would notice.
want(served.oo === 21,
  `the O-O curve is REFUSED by the engine's in-browser split (code ${served.oo}, `
  + `${served.e.holon_bank_pair_n_det(8, 8).toExponential(2)} determinants, predicted `
  + `${served.e.holon_bank_pair_predicted_seconds(8, 8).toFixed(1)} s against a `
  + `${served.e.holon_bank_browser_budget_seconds().toFixed(0)} s load budget)`,
  `expected 21 (PROVENANCE_REFUSED + SplitViolated), got ${served.oo}`);
want(served.oxygen > 0, `the O:2H scene really carries oxygen (${served.oxygen} of ${served.e.holon_atom_count()} atoms)`,
  "every atom reads Z=1, so the composition never reached the engine");
// The count is DERIVED, not merely observed to be non-zero. The scene is 12 atoms with
// every third one oxygen, so it holds 4 O and 8 H, and the triples this engine cannot
// tabulate are exactly (O,H,H), (O,O,H) and (O,O,O):
//
//     4*C(8,2) + C(4,2)*8 + C(4,3)  =  4*28 + 6*8 + 4  =  164
//
// The counter is per force pass rather than cumulative, so this is a property of the
// composition and the identity has to hold exactly. Checking only `> 0` would pass on a
// counter that had lost a whole family of triples, which is precisely the kind of absence
// a threshold cannot see.
const nO = served.oxygen;
const nH = served.e.holon_atom_count() - nO;
const c2 = (n) => (n * (n - 1)) / 2;
const c3 = (n) => (n * (n - 1) * (n - 2)) / 6;
const expectedFence = nO * c2(nH) + c2(nO) * nH + c3(nO);
want(served.fenced === expectedFence,
  `the fence count is exactly the untabulated-triple count for this composition `
  + `(${served.fenced} = ${nO}*C(${nH},2) + C(${nO},2)*${nH} + C(${nO},3))`,
  `counted ${served.fenced}, but ${nO} O and ${nH} H admit ${expectedFence} untabulated `
  + "triples — a family of triples is being missed, or something is being served that "
  + "this build has no surface for");

// The discriminating arm. Without a three-body surface of ANY kind the engine's force pass
// returns early and never reaches these triples, so the counter reads zero — a zero that
// means "never looked", not "nothing to refuse". The page therefore generates the H3
// surface for the O:2H preset too, and this arm is what makes that a requirement rather
// than a habit: if the page stops doing it, `fenced` above becomes an uninformative zero
// and the fence panel starts reporting a clean scene that was never inspected.
const blind = await o2hScene({ withTrimer: false, solveOH: false });
want(blind.fenced === 0,
  "with NO three-body surface the counter reads zero for the wrong reason — which is why "
  + "the page loads H3 even for O-bearing presets",
  `expected the never-looked zero, got ${blind.fenced}; the discrimination this check `
  + "rests on no longer holds and the fence count may be uninformative");

// THE WATER DOOR (FSD-W3 WB-10.7). Before the push the surface is absent — the counter
// above was a measurement of that absence, not a choice — and after it the same scene's
// counter drops by exactly the (O,H,H) family: 4·C(8,2) = 112 triples served, the
// (O,O,H) and (O,O,O) families still fenced. The identity is what makes "served" a fact
// about the force pass rather than a flag on a table.
want(served.e.holon_water_loaded() === 0 && served.e.holon_trimer_surfaces() === 0,
  "before the push the (O,H,H) surface is absent, so the fence above measured an absence");
const waterPush = pushWater(served.e, waterCanonical);
want(waterPush === 1 && served.e.holon_water_loaded() === 1,
  "the committed (O,H,H) table is read through the water door (holon_water_table_load -> 1)",
  `load -> ${waterPush}, loaded -> ${served.e.holon_water_loaded()}`);
// 105,105 SOLVED nodes (the i <= j half of the two O-H sides, times 49 angles) fill a
// 207,025-node symmetric grid (65 x 65 x 49); the export counts the grid the force pass
// reads, the artifact carries the half that was solved
want(served.e.holon_water_nodes() === 207025,
  `the loaded table fills the 207,025-node symmetric grid from its 105,105 solved nodes (${served.e.holon_water_nodes()})`);
// the first value line of the artifact, read here as the parser reads it, is the node the
// door serves at (0,0,0): the bytes went through, not a cached table
const firstHex = waterCanonical.toString("utf8").split("\n").find((l) => l && !l.startsWith("#"));
const firstValue = new DataView(new BigUint64Array([BigInt(`0x${firstHex}`)]).buffer).getFloat64(0, true);
want(served.e.holon_water_node(0, 0, 0) === firstValue,
  `the door serves the artifact's own bytes (node (0,0,0) = ${firstValue.toPrecision(6)} Ha)`,
  `door ${served.e.holon_water_node(0, 0, 0)} vs artifact ${firstValue}`);
served.e.holon_rebase();
for (let f = 0; f < 10; f++) served.e.holon_step_frame(64);
const fencedWithWater = served.e.holon_fence_untabulated();
const expectedWithWater = c2(nO) * nH + c3(nO);
want(fencedWithWater === expectedWithWater,
  `with the water table served the fence count drops to exactly the (O,O,H)+(O,O,O) families `
  + `(${fencedWithWater} = C(${nO},2)*${nH} + C(${nO},3))`,
  `counted ${fencedWithWater}, expected ${expectedWithWater}: (O,H,H) is not being served from the `
  + "pushed table, or another family moved with it");
// a foreign grid line is refused by name and leaves the loaded table untouched
const foreign = Buffer.from(waterCanonical.toString("utf8").replace("NR=65", "NR=165"));
want(pushWater(served.e, foreign) === 0 && served.e.holon_water_loaded() === 1,
  "a table with a foreign grid rule is refused through the door and the loaded one stays");

// WHEN THE ANSWER DEPENDS ON THE ORDER OF THE QUESTION.
//
// `holon_pairs_ready` asks about the pairs THIS SCENE'S ATOMS can meet, so on a box that
// is still all hydrogen it answers about hydrogen. The page asked it before applying the
// composition and got 1 for an O:2H scene whose (O,O) slot is empty and which cannot take
// a single step; it then displayed a frozen box as "SETTLING · 0.0 K → 293.1 K". Both
// readings are pinned here, because a fix that only moves a line is a fix that moves back.
const orderProbe = await freshEngine();
orderProbe.holon_set_dims(1);
orderProbe.holon_bank_clear();
orderProbe.holon_bank_register(1);
orderProbe.holon_bank_register(8);
orderProbe.holon_table_generate(0.6, 12.0, 192);
const readyBefore = orderProbe.holon_pairs_ready();
orderProbe.holon_reset(12);
for (let i = 0; i < orderProbe.holon_atom_count(); i++) {
  orderProbe.holon_set_atom_species(i, i % 3 === 0 ? 8 : 1);
}
const readyAfter = orderProbe.holon_pairs_ready();
want(readyBefore === 1 && readyAfter === 0,
  "readiness is asked AFTER the composition (1 on the all-H box, 0 once oxygen is in it)",
  `before ${readyBefore}, after ${readyAfter} — if these are now equal the page's ordering `
  + "requirement has changed and its 'NOT STEPPING' fence may be unreachable");

// And the page must actually gate its stepping on the second reading, not the first.
want(/served\.stepsAllowed = w\.holon_pairs_ready/.test(appSource)
  && appSource.indexOf("served.stepsAllowed = w.holon_pairs_ready")
     > appSource.indexOf("applyComposition(preset, w.holon_atom_count())"),
  "app.js asks holon_pairs_ready after the composition is applied, not before",
  "the readiness check has moved back above applyComposition, which is the ordering that "
  + "made a frozen scene report itself as settling");

// ------------------------------------------------- 5b. gravity (WB-2.4), now served
//
// This block exists because its entry was DELETED from FENCE_JUSTIFYING_ABSENCES below.
// That list failed when `holon_set_gravity` appeared, which is what it is for; the fence
// came off the page and the obligation moved here. A capability that arrives and is not
// gated is worse than one that was honestly fenced.

const grav = await freshEngine();
grav.holon_set_dims(1);
grav.holon_set_boundary(0);
grav.holon_table_generate(0.6, 12.0, 192);
grav.holon_reset(12);

// The constant is a unit conversion, checked against SI rather than trusted: one G is
// 9.80665 m/s^2, and a_au = a_SI * t_au^2 / a_0.
const gEarth = grav.holon_g_earth();
const BOHR_M = 0.529177210903e-10;
const AU_TIME_S = 2.4188843265857e-17;
const backToSI = (gEarth * BOHR_M) / (AU_TIME_S * AU_TIME_S);
want(Math.abs(backToSI - 9.80665) < 1e-9,
  `one G converts back to 9.80665 m/s² (${gEarth.toExponential(4)} a₀/aut²)`,
  `round-tripped to ${backToSI}`);

// A field of zero holds no potential; a field that is set does. The page's default is
// 1 G, which is far too small to move an f64 ledger, so the conservation arm below runs
// at a field that does real work — a gate that passes because nothing happened is the
// shape this file exists to refuse.
want(grav.holon_gravity() === 0 && grav.holon_e_grav() === 0,
  "a scene with no field holds no gravitational potential");

want(grav.holon_set_gravity(1.0 * gEarth) === 1, "one G is accepted on a walled box");
want(grav.holon_gravity() === gEarth && grav.holon_e_grav() !== 0,
  `1 G is stored and holds real potential (${grav.holon_e_grav().toExponential(3)} Ha)`);

grav.holon_set_gravity(1e18 * gEarth);
grav.holon_rebase();
const wExtBefore = grav.holon_w_ext();
for (let f = 0; f < 200; f++) grav.holon_step_frame(64);
want(grav.holon_energy_gate() === 1,
  `the energy gate closes under a field doing real work (drift ${grav.holon_drift().toExponential(2)} vs bound ${grav.holon_drift_bound().toExponential(2)} Ha)`);
want(grav.holon_momentum_gate() === 1, "the momentum gate closes under gravity");
// The conservative-field obligation, and it runs the OPPOSITE way from the hand's: the
// hand's work IS a receipt, gravity's must not be, or the same joules are counted twice.
want(grav.holon_w_ext() === wExtBefore,
  "gravity posts NOTHING to W_ext — it is conservative, and its energy is V_g",
  `w_ext moved from ${wExtBefore} to ${grav.holon_w_ext()}`);

// THE EXHIBIT, computed rather than quoted. FSD-W1 WB-2.4 stakes ~1e-13 of kT at 1 nm;
// the measured figure is ~4.05e-15, about 25x smaller. The page states the measured one.
const K_B = 3.166811563e-6;
const M_H = 1837.152;
const ratioNm = (M_H * gEarth * (1e-9 / BOHR_M)) / (K_B * 293.15);
want(ratioNm > 1e-15 && ratioNm < 1e-14,
  `1 G is ${ratioNm.toExponential(2)} of kT for a hydrogen atom raised 1 nm — correctly invisible`,
  `ratio ${ratioNm.toExponential(3)}`);

// ------------------------------------------- 5d. WB-2.4c: gravity is a WORLD VECTOR
//
// The scalar door became a wrapper over a vector one. Three things have to hold, and the
// third is the one a conservation check cannot see.

const gv = await freshEngine();
gv.holon_set_dims(1);
gv.holon_set_boundary(0);
gv.holon_table_generate(0.6, 12.0, 192);
gv.holon_reset(12);
const gE = gv.holon_g_earth();
const C = Math.SQRT1_2;

want(gv.holon_set_gravity_vec(gE * C, -gE * C, 0) === 1, "a tilted field is accepted on a walled box");
want(Math.abs(gv.holon_gravity_x() - gE * C) < 1e-30
  && Math.abs(gv.holon_gravity_y() + gE * C) < 1e-30
  && gv.holon_gravity_z() === 0,
  "the field vector reads back component-wise",
  `(${gv.holon_gravity_x()}, ${gv.holon_gravity_y()}, ${gv.holon_gravity_z()})`);
want(Math.abs(gv.holon_gravity() / gE - 1) < 1e-15,
  "holon_gravity reports the vector's MAGNITUDE (a tilted 1 G is still 1 G)",
  `magnitude ${gv.holon_gravity()} against g ${gE}`);

// THE DIRECTION HAS TO MATTER, and only a strong field over real time can show it. The
// engine-side test measures 21.5 bohr of x-separation against 3.4e-12 for a projected
// vector; here the bar is one bohr for the same reason it is there — a threshold at zero
// passes on float divergence, which is how the engine test's first version was found unable
// to fail.
async function tiltRun(gx, gy) {
  const e = await freshEngine();
  e.holon_set_dims(1);
  e.holon_set_boundary(0);
  e.holon_table_generate(0.6, 12.0, 192);
  e.holon_reset(12);
  e.holon_set_gravity_vec(gx, gy, 0);
  e.holon_rebase();
  for (let f = 0; f < 200; f++) e.holon_step_frame(64);
  return e;
}
const big = 1e18 * gE;
const tilted = await tiltRun(big * C, -big * C);
const down = await tiltRun(0, -big);
let sep = 0;
for (let i = 0; i < tilted.holon_atom_count(); i++) {
  sep = Math.max(sep, Math.abs(tilted.holon_atom_x(i) - down.holon_atom_x(i)));
}
want(sep > 1.0,
  `a tilted field really tilts — ${sep.toFixed(2)} bohr of x-separation from the vertical one`,
  `only ${sep.toExponential(3)} bohr; below an interatomic distance this is float divergence, `
  + "not direction, and the vector is being projected somewhere");
want(tilted.holon_energy_gate() === 1 && tilted.holon_w_ext() === 0,
  "a tilted field conserves and posts nothing to W_ext");

// The periodic refusal is BROWSER-REACHABLE now that boundary mode 2 selects Periodic.
// It was not when gravity shipped, and the page's fence said so; this is the check that
// the page's claim tracks the engine rather than the other way round.
const per = await freshEngine();
per.holon_set_dims(1);
per.holon_table_generate(0.6, 12.0, 192);
per.holon_set_boundary(2);
want(per.holon_gravity_available() === 0,
  "a wrapping box reports the field unavailable, and mode 2 reaches it from the ABI");
for (const [gx, gy, gz] of [[gE, 0, 0], [0, -gE, 0], [0, 0, gE]]) {
  const code = per.holon_set_gravity_vec(gx, gy, gz);
  want(code === 80,
    `a wrapping box refuses the field along (${gx ? "x" : gy ? "y" : "z"}) with code ${code}`,
    `expected 80 (GRAVITY_REFUSED + PeriodicBox), got ${code}`);
}

// ------------------------------------------- 5e. WB-2.2: the control is the box
const bx = await freshEngine();
bx.holon_set_dims(1);
bx.holon_set_boundary(0);
bx.holon_table_generate(0.6, 12.0, 192);
bx.holon_reset(12);
bx.holon_set_gravity_vec(0, -1e18 * gE, 0);
bx.holon_rebase();
for (let f = 0; f < 50; f++) bx.holon_step_frame(64);
const wBox = bx.holon_w_ext();
const gravBox = bx.holon_e_grav();
want(bx.holon_box_scale(0.9) === 1, "the box compresses through the engine's own door");
want(bx.holon_e_grav() !== gravBox,
  "an affine compression under gravity changes the gravitational potential");
want(bx.holon_w_ext() !== wBox, "the compression's cost is posted to the ledger");
for (let f = 0; f < 200; f++) bx.holon_step_frame(64);
want(bx.holon_energy_gate() === 1,
  `the energy gate closes after compressing under gravity (drift ${bx.holon_drift().toExponential(2)} vs bound ${bx.holon_drift_bound().toExponential(2)})`);
want(bx.holon_box_scale(0) !== 1 && bx.holon_box_scale(-1) !== 1,
  "a nonsense scale factor is refused rather than applied");

// ---------------------------------------------------------------- 6. determinism (WB-5.4)

// `w` already carries the H3 surface from the pure-H section above, and
// `holon_bank_clear` clears the PAIR bank only — the three-body surface survives it. So
// the digest runs with the three-body term live without paying for a fourth 14,157-node
// generation, and it asserts that rather than assuming it, because a determinism check
// that silently lost the three-body term would be checking the easy half.
function digest() {
  if (w.holon_trimer_loaded() !== 1) throw new Error("digest() lost the H3 surface");
  w.holon_bank_clear();
  w.holon_bank_register(1);
  w.holon_table_generate(0.6, 12.0, 192);
  w.holon_reset(12);
  for (let f = 0; f < 100; f++) w.holon_step_frame(64);
  const n = w.holon_atom_count();
  const buf = new Float64Array(n * 4);
  for (let i = 0; i < n; i++) {
    buf[i * 4] = w.holon_atom_x(i);
    buf[i * 4 + 1] = w.holon_atom_y(i);
    buf[i * 4 + 2] = w.holon_atom_z(i);
    buf[i * 4 + 3] = w.holon_atom_speed(i);
  }
  const b = new Uint8Array(buf.buffer);
  let h = 0x811c9dc5;
  for (let i = 0; i < b.length; i++) { h ^= b[i]; h = Math.imul(h, 0x01000193) >>> 0; }
  return h.toString(16).padStart(8, "0");
}
const dA = digest();
const dB = digest();
want(dA === dB, `the seeded scene replays bit-identically in this device class (${dA})`,
  `run A ${dA}, run B ${dB}`);

// ------------------------------------------- 6b. every RECORD citation still resolves
//
// The water-story panel is the one place this page shows numbers it did not compute. Each
// carries `artifact:line`, and this reads that file and requires the number to BE there.
//
// It is the check my own house rules have been asking for and I had not written: three
// claims in one day once rested on documents nobody had written, and a citation is exactly
// the shape that rots silently — the page keeps displaying 893.8 fs long after the census
// is re-run and prints something else. Two prongs, because a citation can fail two ways:
// the artifact can vanish, and the number can move inside it.

// `repoRoot` is resolved at the top of this file. It used to be resolved here, and moved
// when the export-contract section started citing the FSD: a `let` read before its
// initialiser runs is a TDZ throw, which this gate would have reported as "coverage
// unknown" rather than as the ordering mistake it is.

const recordBlock = appSource.match(/const RECORD = \{([\s\S]*?)\n\};/);
want(recordBlock !== null, "the page's RECORD block is where the gate expects it");
if (recordBlock) {
  // `match` is optional and overrides `value` for the lookup. It exists because a DISPLAY
  // figure and a CHECKABLE one are not always the same string: "0" is the honest thing to
  // show for a solver counter and is useless to look for, since it occurs on nearly every
  // line of nearly every file. A citation that passes without establishing anything is the
  // failure this whole block exists to prevent, so the block must not commit it.
  const entries = [...recordBlock[1].matchAll(
    /value:\s*"([^"]+)",[\s\S]*?cite:\s*"([^"]+)"(?:,\s*(?:\/\/[^\n]*\n\s*)*match:\s*"([^"]+)")?/g)];
  want(entries.length > 0, "the RECORD block carries at least one cited figure");
  // The repository root, FOUND rather than assumed. `join(here, "..", "..")` is right only
  // while this file sits exactly two levels down, and it silently resolves to the wrong
  // directory the moment the gate is run from a copy — which made every mutation test of
  // this block fail with "artifact does not exist" instead of the defect it was probing.
  // A check whose failures all look the same cannot tell you which one fired.
  // (repoRoot is computed once, above.)
  for (const [, value, cite, override] of entries) {
    const needle = override ?? value;
    const [relPath, lineNo] = cite.split(":");
    const abs = join(repoRoot, relPath);
    let text = null;
    try { text = readFileSync(abs, "utf8"); } catch { /* reported below */ }
    if (text === null) {
      no(`RECORD cites ${relPath}, which does not exist`,
        "a published page must not cite an artifact a clean checkout does not have; if the "
        + "run is banked but uncommitted, the figure does not go on the page yet");
      continue;
    }
    const lines = text.split("\n");
    const line = lines[Number(lineNo) - 1] ?? "";
    // WHAT TO LOOK FOR, and this rule has been wrong twice.
    //
    // Try the LITERAL value first, then the digit-stripped form. Stripping alone was wrong
    // for a non-numeric value ("NOT CLOSED" strips to the empty string, and
    // `line.includes("")` is true of every line — a vacuous pass on the very citation it
    // was checking). Stripping-with-a-fallback was still wrong for a hex commit hash
    // ("21e6be3" strips to "2163", which is length > 0 and is not in the file, so a
    // perfectly good citation failed). Literal-then-stripped handles all three: the hash
    // and "NOT CLOSED" match literally, and "893.8 fs" matches after its unit comes off,
    // because the artifact writes it in a fixed-width column with no unit.
    const stripped = needle.replace(/[^0-9.]/g, "");
    const candidates = [needle, stripped].filter((c) => c.length > 0);
    const bare = candidates.find((c) => line.includes(c))
      ?? candidates.find((c) => text.includes(c))
      ?? needle;
    if (line.includes(bare)) {
      ok(`RECORD "${value}" is on ${cite}`);
    } else if (text.includes(bare)) {
      no(`RECORD "${value}" is in ${relPath} but NOT on line ${lineNo}`,
        `line ${lineNo} reads: ${line.trim().slice(0, 90)}`);
    } else {
      no(`RECORD "${value}" is not in ${relPath} at all`,
        "the artifact has been re-run or replaced and the page is quoting a number that is "
        + "no longer in the record");
    }
  }
}

// And the control's figure must NOT be on the page while its artifact is uncommitted.
// This is the inverted twin of the check above: there, a citation must resolve; here, an
// UNCITED number must be absent. Without it the honest omission decays into an oversight
// the moment someone pastes the figure in.
const html6b = readFileSync(join(here, "index.html"), "utf8");

// TRACKED, not merely PRESENT — and the difference is the whole check. An untracked file
// exists on the author's disk and does not exist in CI's clean checkout, so testing
// `readFileSync` succeeds locally and passes vacuously exactly where the defect lives.
// That was this check's first version, and its plant sailed through.
//
// `git ls-files --error-unmatch` FAILS rather than skips when git is absent: a gate that
// quietly passes when it cannot check is the shape this file exists to refuse.
const de4OffTracked = (() => {
  try {
    execFileSync("git", ["ls-files", "--error-unmatch",
      "conformance/water_observatory/census_de4_off.log"],
      { cwd: repoRoot, stdio: "pipe" });
    return true;
  } catch (e) {
    if (e && e.code === "ENOENT") {
      no("the RECORD tracking check needs git, which is not on PATH (NOT skipped)");
      return true; // already reported; do not double-fail below
    }
    return false;
  }
})();
want(de4OffTracked || !/923\.9/.test(html6b + appSource),
  "the control's figure stays off the page until its artifact is COMMITTED",
  "923.9 fs appears on the page while census_de4_off.log is untracked — it exists on this "
  + "disk and not in a clean checkout, so no reader could verify it");

// ------------------------------------------- 6c. the scale ladder wears real fences
//
// Two properties, and the second is the fence law applied to this page rather than
// restated on it: every band's citation must resolve to the FSD line it claims, and every
// FENCED band must carry an OWNER and an EXIT. A fence without those is a shrug with a
// border around it, and it is exactly what the ladder would decay into if the coarse
// bands stayed un-owned long enough for someone to stop noticing.

const ladderBlock = appSource.match(/const LADDER = \[([\s\S]*?)\n\];/);
want(ladderBlock !== null, "the page's LADDER block is where the gate expects it");
if (ladderBlock) {
  // ONE parse, by splitting on entry boundaries rather than by a field-order regex.
  //
  // The previous version matched `band ... state ... cite` and required `cite` to be the
  // LAST field before the closing brace. Adding a `certificate` field after it silently
  // broke the match, the non-greedy span ran on into the NEXT entry, and the gate reported
  // three bands and paired the molecular band's name with its neighbour's citation. A
  // checker that assumes field order is a checker that breaks when the thing it checks
  // grows — which is exactly when you need it.
  const bandBlocks = ladderBlock[1]
    .split(/\n  \{/)
    .map((b) => b.trim())
    .filter((b) => /^\s*band: "/.test(b) || /\bband: "/.test(b));
  const field = (block, name) => {
    const m = block.match(new RegExp(`${name}: "((?:[^"\\\\]|\\\\.)*)"`));
    return m ? m[1] : null;
  };
  const bands = bandBlocks.map((block) => ({
    block,
    band: field(block, "band"),
    state: field(block, "state"),
    cite: field(block, "cite"),
    certificate: field(block, "certificate"),
    measuredBy: field(block, "measuredBy"),
    positiveCite: field(block, "positiveCite"),
    readoutCite: field(block, "readoutCite"),
    declaredCite: field(block, "declaredCite"),
    buildCite: field(block, "buildCite"),
    ganttCite: field(block, "ganttCite"),
    lengthM: Number((block.match(/lengthM: ([0-9.e+-]+)/) || [, NaN])[1]),
    liveWhen: [...(block.match(/liveWhen: \[([\s\S]*?)\],/) || [, ""])[1]
      .matchAll(/"([a-z0-9_]+)"/g)].map((m) => m[1]),
  }));

  // ---- EVERY BAND IN §11.2's TABLE IS ON THE PAGE ----------------------------
  //
  // FSD-W3 §11.2 is the spec of record and its content is that the ladder runs from 1 km to
  // the nucleus with NOTHING MISSING: every band present, each one live or fenced. A count
  // alone would pass on seven bands with the wrong names, so the names are pinned — and the
  // one row §11.2 carries that must NOT be here is pinned too, in the other direction.
  //
  // "below the nucleus" is the gauge vacuum (W2), which §11.2 marks NOT ON THIS PAGE YET and
  // the operator sequenced after this build. A band drawn for unscheduled work is a promise
  // wearing a fence's clothes, so its ABSENCE is the checked property.
  const LADDER_BANDS = [
    "the cube", "fluid element", "H-bond network", "molecular",
    "atom", "nucleus", "the fold below the atom",
  ];
  const names = bands.map((b) => b.band);
  want(bands.length === LADDER_BANDS.length,
    `the ladder carries all ${LADDER_BANDS.length} bands of FSD-W3 §11.2 (found ${bands.length})`,
    `parsed: ${names.join(", ")}`);
  for (const wanted of LADDER_BANDS) {
    want(names.includes(wanted), `§11.2's "${wanted}" band is present on the page`,
      "every band in the table is on the page, live or fenced — nothing in between and "
      + "nothing missing");
  }
  want(!names.some((n) => /below the nucleus/i.test(n) && n !== "the fold below the atom"),
    "the gauge-vacuum row §11.2 marks NOT ON THIS PAGE YET is not drawn as a band",
    "W2 is sequenced after this build; a band on the page for unscheduled work is a promise "
    + "wearing a fence's clothes");

  // ---- THE LADDER IS ORDERED, CUBE DOWN TO NUCLEUS ---------------------------
  //
  // The operator's order is a zoom axis, and an axis whose rungs are out of order is not
  // one. Strict monotonicity is the checkable form: each band's own scale is smaller than
  // the one above it, so the page cannot silently grow a rung in the middle of the ladder.
  const lengths = bands.map((b) => b.lengthM);
  want(lengths.every((v) => Number.isFinite(v) && v > 0),
    "every band declares a positive scale length",
    `lengthM values: ${lengths.join(", ")}`);
  const descendingOrder = lengths.every((v, k) => k === 0 || v < lengths[k - 1]);
  want(descendingOrder,
    "the ladder runs from the cube DOWN to the nucleus (scale strictly decreasing)",
    `order as declared: ${names.map((n, k) => `${n} ${lengths[k]}`).join(" · ")}`);
  want(names[0] === "the cube" && names[names.length - 1] === "the fold below the atom",
    "the ladder's ends are the 1 km cube and the fold below the atom",
    `first "${names[0]}", last "${names[names.length - 1]}"`);

  // ---- EACH BAND'S STATE IS THE ONE §11.2 GIVES IT ---------------------------
  //
  // The count and the names would both pass on a ladder with every band fenced, which is
  // the ladder §11.2 does not describe. So the state is pinned per band, and the two fine
  // ones are pinned TWICE: their source state is `export-gated` (the flip is the artifact's,
  // not a word's) AND §11.2 says they are LIVE, so the artifact must actually serve them.
  // Failing that second half is what "the page shows a fence where the spec shows a band"
  // looks like from the outside, and nothing else here would say it.
  const SPEC_STATE = {
    "the cube": "fenced", "fluid element": "fenced", "H-bond network": "fenced",
    "molecular": "live", "atom": "export-gated", "nucleus": "export-gated",
    "the fold below the atom": "fenced",
  };
  for (const b of bands) {
    const expect = SPEC_STATE[b.band];
    if (!expect) continue;
    want(b.state === expect, `band "${b.band}" is ${expect}, as §11.2 has it`,
      `the page says "${b.state}". §11.2 is the spec of record for this ladder; a band that `
      + "disagrees with it is either the spec moving or the page drifting, and one of the "
      + "two has to be edited");
    if (expect === "export-gated") {
      const unserved = b.liveWhen.filter((n) => typeof w[n] !== "function");
      want(unserved.length === 0,
        `band "${b.band}" is LIVE on this artifact, as §11.2 says it is`,
        `this artifact does not serve ${unserved.join(", ")}, so the band renders FENCED — `
        + "PENDING while §11.2 says LIVE. Either the wasm is behind the spec or a name is "
        + "wrong; the page is doing the right thing with the artifact it has");
    }
  }

  for (const { block: whole, band, state, cite } of bands) {
    const [relPath, lineNo] = cite.split(":");
    let text = null;
    try { text = readFileSync(join(repoRoot, relPath), "utf8"); } catch { /* below */ }
    if (text === null) { no(`band "${band}" cites ${relPath}, which does not exist`); continue; }
    const line = (text.split("\n")[Number(lineNo) - 1] ?? "").toLowerCase();
    // The band name must actually appear on the line it cites — otherwise the citation is
    // decoration, and the page could drift a whole band away from the spec unnoticed.
    want(line.includes(band.toLowerCase()),
      `band "${band}" is on ${cite}`,
      `line ${lineNo} reads: ${(text.split("\n")[Number(lineNo) - 1] ?? "").trim().slice(0, 80)}`);

    // A band that is not LIVE is fenced, whatever its warrant would be: the coarse bands
    // wait on a closure certificate and the fine ones wait on their exports, and BOTH owe
    // the reader an owner and an exit. Scoping this to `state === "fenced"` would have let
    // the two new bands carry a pending state with nobody named, which is the shrug the
    // fence law exists to forbid.
    if (state === "fenced" || state === "export-gated") {
      const hasOwner = /owner:\s*"[^"]+"/.test(whole);
      const hasExit = /exit:\s*"[^"]{40,}"/.test(whole);
      want(hasOwner && hasExit,
        `${state} band "${band}" carries an owner and an exit`,
        `owner ${hasOwner ? "present" : "MISSING"}, substantive exit ${hasExit ? "present" : "MISSING"} `
        + "— the fence law requires both, and an exit too short to say anything is not one");
    }
  }

  // ---- THE FINE BANDS FLIP ON EXPORTS, AND NAME EVERY ONE --------------------
  //
  // The coarse bands' warrant is a node-G closure certificate, gated below. The fine bands
  // hold none and are not entitled to one — a closure certificate certifies a COARSE view
  // of the dynamics beneath a band and there is no coarse view at an atom. Their warrant is
  // the artifact: `liveWhen` names the exports that must resolve, and until they do the band
  // draws no digits.
  //
  // The failure this catches is a band fenced on an export nobody named — indistinguishable
  // on screen from a band waiting for work in progress, and permanent.
  for (const { band, state, liveWhen } of bands) {
    if (state !== "export-gated") continue;
    want(liveWhen.length > 0,
      `export-gated band "${band}" names the exports it is waiting for`,
      "a band gated on nothing can never flip, and its fence has no exit a reader can check");
    const strays = liveWhen.filter((n) => !pending.includes(n));
    want(strays.length === 0,
      `every export "${band}" waits on is on PENDING_EXPORTS (${liveWhen.length})`,
      strays.length ? `not on the pending list: ${strays.join(", ")} — a name that is on `
        + "neither list is a name nothing is building" : undefined);
  }
  // AND THE OTHER DIRECTION: every pending export is claimed by some band or by the
  // readouts card. An export on the list that nothing waits for is a fence with no panel
  // behind it, and it would keep looking like scheduled work forever.
  const claimed = new Set(bands.flatMap((b) => b.liveWhen));
  for (const name of pending) {
    const inLadder = claimed.has(name);
    const inCard = new RegExp(`"${name}"`).test(appSource.slice(appSource.indexOf("DESCENT_FIELDS")))
      || new RegExp(`hasExport\\("${name}"\\)`).test(appSource);
    want(inLadder || inCard,
      `pending export ${name} is claimed by a band or by a readout row`,
      "nothing on the page waits for this export, so its absence fences nothing and its "
      + "arrival would change nothing");
  }
  // ---- THE FLIP IS MECHANICAL, IN BOTH DIRECTIONS -----------------------------
  //
  // §9c's ladder unlocks band by band as node G's rungs certify. The staking is that a
  // band flips fenced -> live ONLY on a banked certificate whose citation RESOLVES, and
  // this is that rule as code rather than as a habit. Two directions, because one alone
  // is a door with a hinge and no latch:
  //
  //   live  => a certificate is named AND resolves. Without this, flipping a band is
  //            editing one word, and the word would be believed.
  //   resolves => live. Without this, a rung could land, its certificate could be wired
  //            in, and the band could sit fenced indefinitely with nobody told — the
  //            same absence-shaped rot the gravity fence had, pointed the other way.
  //
  // `cite` is deliberately NOT sufficient for either: it points at the FSD line that says
  // a band should be live, which is a PLAN. The certificate is a VERDICT. The molecular
  // band was live on the plan alone until this check was written, which is why the rule
  // had to bind the band that was already flipped before it could be trusted on the ones
  // that are not.
  for (const { block, band, state, certificate } of bands) {
    const isLive = state === "live";
    // WHICH NODE certified it. Only node G's certificates are band states: a node-G
    // certificate is a certified coarse view of the dynamics beneath the band. Node LG's
    // lattice-gas tier is certified on its OWN dynamics — supporting machinery and
    // research content — so an LG bank must NOT fire the flip-owed direction. Without
    // this the gate would read any line containing "CERTIFIED" as a band's warrant and
    // demand a flip that §9c forbids.
    const certNode = (block.match(/certNode: "([^"]+)"/) || [, null])[1];
    let certResolves = false;
    if (certificate) {
      const [relPath, lineNo] = certificate.split(":");
      try {
        const text = readFileSync(join(repoRoot, relPath), "utf8");
        const line = text.split("\n")[Number(lineNo) - 1] ?? "";
        // A certificate is a VERDICT, so the cited line must actually carry one. "CERTIFIED"
        // is the census's own word; a citation to a line that merely mentions the band would
        // pass a weaker check while establishing nothing.
        const positive = carriesPositiveVerdict(line);
        const negated = /\bCERTIFIED\b/i.test(line) && !positive;
        certResolves = positive && certNode === "G";
        if (negated) {
          // A NEGATED verdict cited as a certificate is a wiring mistake worth naming —
          // and naming it must NOT be done by demanding a flip. Rung 2 banking branch (d)
          // is a legitimate state; the page's own fluid band cites that document through
          // `measuredBy`, which is the right field for it.
          no(`band "${band}" cites a NOT-CERTIFIED verdict as a certificate (${certificate})`,
            `line ${lineNo} negates its own verdict: ${line.trim().slice(0, 70)} — cite a `
            + "banked verdict through `measuredBy`, never as the certificate that flips a band");
        }
        if (positive && certNode !== "G") {
          // Not a failure: a real certificate from a node that does not confer a band
          // state. Reported so the distinction is visible rather than silent.
          ok(`band "${band}" cites a node-${certNode ?? "?"} certificate, which is not a band state`);
        }
        if (!certResolves) {
          if (!/\bCERTIFIED\b/i.test(line)) no(`band "${band}" cites a certificate at ${certificate} that carries no verdict`,
            `line ${lineNo} reads: ${line.trim().slice(0, 90)}`);
        }
      } catch {
        no(`band "${band}" cites a certificate in ${relPath}, which does not exist`);
      }
    }
    // Only the direction that BITES for this band is emitted. Reporting "FENCED band X does
    // not hold a certificate" about a live band is a check that passes by not applying, and
    // a log full of those is how a reader stops reading the log.
    if (isLive) {
      want(certResolves,
        `LIVE band "${band}" is backed by a resolving certificate`,
        certificate
          ? "the certificate does not resolve to a line carrying a verdict"
          : "no `certificate` field — a band may not be live on the SPEC line alone; the FSD "
            + "saying a band should be live is a plan, and the flip needs a banked verdict");
    } else {
      want(!certResolves,
        `FENCED band "${band}" does not already hold a resolving certificate`,
        "this band's certificate RESOLVES while it is still fenced — the rung has landed "
        + "and the flip is owed: set state to \"live\" and un-fence the panel");
    }
  }

  // ---- A MEASURED FENCE MUST BE CHECKABLE ------------------------------------
  //
  // Rung 2 came back NOT CERTIFIED with numbers rather than a shrug, and the fluid band's
  // fence now carries them. Numbers on a page are a liability unless they resolve: these
  // are cited to RUNG2_RESULTS.md and read out of it here, exactly like the RECORD block.
  // A fence that quotes a measurement nobody checks is a longer sentence, not a better one.
  for (const { band, measuredBy, positiveCite, readoutCite, declaredCite, buildCite, ganttCite } of bands) {
    for (const [label, cite] of [
      ["measured exit", measuredBy], ["positive finding", positiveCite],
      ["readout grant", readoutCite], ["declared-input rule", declaredCite],
      ["build row", buildCite], ["GANTT row", ganttCite],
    ]) {
      if (!cite) continue;
      const [relPath, lineNo] = cite.split(":");
      try {
        const text = readFileSync(join(repoRoot, relPath), "utf8");
        const line = text.split("\n")[Number(lineNo) - 1] ?? "";
        want(line.trim().length > 0,
          `band "${band}" ${label} cites a real line at ${cite}`,
          `line ${lineNo} of ${relPath} is empty`);
      } catch {
        no(`band "${band}" ${label} cites ${relPath}, which does not exist`);
      }
    }
  }

  // The two figures the fluid band's fence rests on, pinned against their artifact. If
  // rung 2 re-runs and these move, the page must move with them rather than keep quoting
  // a superseded measurement — which is the failure the RECORD gate exists to prevent,
  // arriving in a fence instead of a figure.
  const r2 = (() => {
    try { return readFileSync(join(repoRoot, "conformance/water_observatory/RUNG2_RESULTS.md"), "utf8"); }
    catch { return null; }
  })();
  want(r2 !== null, "RUNG2_RESULTS.md is in the tree");
  if (r2) {
    // REQUIRED, not conditional — and the first version of this was conditional, which
    // made it evadable by the exact edit it exists to catch. It read "IF the page quotes
    // 5.95e6 THEN the artifact must have it", so changing the page's figure to 9.99e9
    // simply made the check not apply, and the plant sailed through. Both sides are
    // demanded now: the page MUST carry the figure and the artifact MUST still contain it,
    // so the check fails whichever of the two moves without the other.
    const fluid = bands.find((b) => b.band === "fluid element");
    want(!!fluid, "the fluid-element band is present to check");
    for (const fig of ["5.95e6", "+0.598"]) {
      const onPage = !!fluid && fluid.block.includes(fig);
      const inArtifact = r2.includes(fig.replace("+", ""));
      want(onPage && inArtifact,
        `the fluid band quotes ${fig} and RUNG2_RESULTS.md still carries it`,
        onPage
          ? `the page quotes ${fig} but the artifact no longer contains it — rung 2 has `
            + "re-run and the fence is quoting a superseded measurement"
          : `the page no longer quotes ${fig}. If rung 2 re-measured, move the figure here `
            + "too; if it was simply edited away, the fence has lost the number that made "
            + "it a measured fence rather than a state");
    }
  }

  // THE FOLD'S QUOTED FIGURE, pinned to the register row it cites — same rule as the fluid
  // band's, and for the same reason: a fence that quotes a measurement nobody checks is a
  // longer sentence, not a better one. If GF2's first rung re-reads, the page must move with
  // it rather than keep quoting a superseded number.
  const fold = bands.find((b) => b.band === "the fold below the atom");
  want(!!fold, "the fold band is present to check");
  if (fold) {
    const tiers = (() => {
      try { return readFileSync(join(repoRoot, "TIERS.md"), "utf8"); } catch { return null; }
    })();
    want(tiers !== null, "TIERS.md is in the tree");
    if (tiers) {
      const onPage = fold.block.includes("0.6%");
      const line = tiers.split("\n")[Number(fold.positiveCite.split(":")[1]) - 1] ?? "";
      want(onPage && line.includes("0.6%"),
        "the fold band quotes 0.6% and TIERS.md's hadron row still carries it",
        onPage
          ? "the page quotes 0.6% and the cited row no longer does — GF2's first rung has "
            + "re-read and the fence is quoting a superseded measurement"
          : "the page no longer quotes the figure that made this a measured fence rather "
            + "than a state");
    }
  }

  // EVERY FENCED BAND NAMES A BUILD IN PROGRESS, not just a condition (operator's law: a
  // fence is a bug under repair, never content). The discriminator is deliberately crude —
  // present-tense build language — because that is what a text check can see. It was the
  // TENSE that was wrong; the numbers were always right.
  //
  // SCOPED TO BANDS THAT ARE ACTUALLY FENCED, and that scoping is a correction rather than
  // a loophole. An export-gated band whose exports have landed is LIVE and owes no debt;
  // demanding it name a build in progress forces a sentence that was true this morning and
  // is false now. That is exactly what happened — the atom band's exit said its exports
  // "are in build in holon-render now" for as long as it took them to land — so the rule is
  // stated in BOTH directions here, and the stale half is the one nothing was checking.
  const servedNow = (b) => b.state === "live"
    || (b.state === "export-gated" && b.liveWhen.length > 0
      && b.liveWhen.every((n) => typeof w[n] === "function"));
  const BUILD_TENSE = /\b(is|are) (being )?(built|in build)\b|\bin build\b|\bgoes live as\b/i;
  for (const b of bands) {
    if (b.state !== "fenced" && b.state !== "export-gated") continue;
    if (servedNow(b)) {
      want(!BUILD_TENSE.test(b.block),
        `live band "${b.band}" no longer describes its exports as work in progress`,
        "this band's exports resolve in the committed artifact, so it renders LIVE — and its "
        + "exit still promises a build. A fence that outlives its debt is the F-2 shape: the "
        + "page telling viewers about an absence that ended");
      continue;
    }
    want(BUILD_TENSE.test(b.block),
      `${b.state} band "${b.band}" names a build in progress, not just a condition`,
      "the fence must say what work is paying the debt, in the present tense — a band whose "
      + "exit names only a state is describing a refusal rather than a repair");
  }

  // EXACTLY ONE BAND IS DECLARED LIVE IN THE SOURCE, and the claim is narrower than it was
  // — deliberately, because the ladder grew two bands whose state is not in the source at
  // all. What this still forbids is the thing it was written to forbid: a COARSE chart
  // declared live that this engine does not have, flipped by editing one word.
  //
  // The fine bands cannot be flipped that way. Their state is computed from the artifact
  // every frame by `bandLiveness`, which is exercised in both directions below, so there is
  // no word here to edit — which is why they are excluded from this count rather than
  // counted and forgiven.
  const live = bands.filter((b) => b.state === "live").length;
  const gated = bands.filter((b) => b.state === "export-gated").length;
  want(live === 1,
    `exactly one band is declared LIVE in the source (${live}); ${gated} flip from the artifact`,
    "more than one live band means a coarse chart is being served that this engine does "
    + "not have, which is the tier-faking the ladder exists to forbid");
  const known = bands.filter((b) => ["live", "fenced", "export-gated"].includes(b.state)).length;
  want(known === bands.length,
    "every band's state is one of live, fenced or export-gated",
    `states: ${bands.map((b) => `${b.band}=${b.state}`).join(", ")} — a state the renderer `
    + "does not know renders as a fenced band with no fence, which is a blank row");
}

// ------------------------------- 6c1. the fine bands' flip is the ARTIFACT's, not a word's
//
// `bandLiveness` is the whole rule for the atom and nucleus bands: live when every export
// they name resolves, FENCED — PENDING naming the missing ones otherwise. It is lifted out
// of the page and run here rather than re-implemented, for the same reason `acuityPopulation`
// is — a second implementation in the checker would drift from the one that ships and would
// be testing itself.
//
// BOTH DIRECTIONS, because one alone is a door with a hinge and no latch. Forward: a missing
// export must fence the band and NAME what is missing, so the page cannot draw a digit it
// does not have. Reverse: with every export present the band must go live with no edit to
// app.js, so a rung cannot land and leave the band fenced with nobody told — the
// absence-shaped rot the gravity fence had, pointed the other way.
const livenessSrc = appSource.match(/function bandLiveness\(liveWhen, has\) \{\n([\s\S]*?)\n\}/);
want(livenessSrc !== null, "the page implements bandLiveness");
if (livenessSrc) {
  let liveness;
  try {
    liveness = new Function("liveWhen", "has", livenessSrc[1]);
  } catch (e) {
    no("bandLiveness's body could not be reconstructed for testing", String(e));
  }
  if (liveness) {
    const three = ["holon_atom_band_energy", "holon_atom_band_exit", "holon_law_probe"];
    const all = liveness(three, () => true);
    want(all.live === true && all.missing.length === 0,
      "with every export present the band flips LIVE with no edit to the page",
      `got live=${all.live}, missing=${JSON.stringify(all.missing)}`);
    const one = liveness(three, (n) => n !== "holon_atom_band_exit");
    want(one.live === false && one.missing.length === 1 && one.missing[0] === "holon_atom_band_exit",
      "one missing export fences the band and NAMES the export",
      `got live=${one.live}, missing=${JSON.stringify(one.missing)} — a fence that cannot `
      + "say what it is waiting for has no exit a reader can check");
    const none = liveness(three, () => false);
    want(none.live === false && none.missing.length === 3,
      "with none present all three are named, not just the first");
    // AND THE DEGENERATE CASE, which is the one that would pass by not applying: a band
    // that names no export must NOT read live. Without this, deleting a `liveWhen` list
    // would flip its band live and every other check here would still be green.
    const empty = liveness([], () => true);
    want(empty.live === false,
      "a band naming NO exports does not read live — an empty gate is not a passed gate",
      "vacuous truth over an empty list would flip a band on the deletion of its own warrant");
  }
}

// ------------------------------------------- 6c2. the acuity law seeds at ONE
//
// §9c's acuity law is what makes a 1 km cube of ~3e31 molecules cheap: a band's population
// is what the view can distinguish, seeded at one. Both of the law's stated figures are
// checked here against the page's own implementation, because the FSD admits two readings
// that differ by nine orders and the page had to choose one.
//
//   - span = the band's own scale  -> exactly 1 (the pinned seed)
//   - span = 10x that scale        -> 1000 ("thousands at full molecular zoom")
//   - span below the band's scale  -> 0 (nothing to allocate)
//
// The rejected reading — the paragraph's "(a molecule at a pixel)" parenthetical — puts
// 3.0e9 molecules in view at the seed, which contradicts the same paragraph's "ONE". The
// arithmetic is pinned here so the page cannot drift onto it silently.
// The page's OWN function is executed, not a copy of it — a second implementation in the
// checker would drift from the one that ships and would be testing itself. The body is
// captured between the brace that opens it and the one that closes it at column zero.
const acuitySrc = appSource.match(
  /function acuityPopulation\(viewSpanM, lengthM\) \{\n([\s\S]*?)\n\}/);
want(acuitySrc !== null, "the page implements acuityPopulation");
if (acuitySrc) {
  let acuity;
  try {
    acuity = new Function("viewSpanM", "lengthM", acuitySrc[1]);
  } catch (e) {
    no("acuityPopulation's body could not be reconstructed for testing", String(e));
  }
  if (acuity) {
  const L = 3.0e-10;
  want(acuity(L, L) === 1, "at the band's own scale the population is the ONE pinned seed",
    `got ${acuity(L, L)}`);
  want(acuity(10 * L, L) === 1000, "ten times that scale admits a thousand — the law's own 'thousands'",
    `got ${acuity(10 * L, L)}`);
  want(acuity(L / 2, L) === 0, "below the band's scale there is nothing to allocate",
    `got ${acuity(L / 2, L)}`);
  // And the number the law exists to avoid: a kilometre of water, never enumerated.
  want(acuity(1e3, L) > 1e30,
    "the law does not pretend a 1 km view is cheap — it reports the 1e31 it refuses to allocate",
    `got ${acuity(1e3, L).toExponential(2)}`);
  }
}

// ------------------------- 6c4. the ladder's readouts: every digit names its source
//
// WB-7 at the bottom of the ladder. The nucleus band is the hardest place on this page to
// obey it — three of its numbers are MEASURED INPUTS the Hamiltonian never computes
// (WB-1.7), three are waiting on exports WB-10.1 is building, and two of the cube band's
// are page arithmetic — so "a number either traces or it is fenced" needs a third and
// fourth word, and each of them needs a checker or it is a promise.
//
// `DESCENT_FIELDS` is that contract as data: one entry per row, each naming its source in
// one of exactly four forms. Everything below resolves those names against the artifact,
// the committed tables and the markup, in both directions.
const descBlock = appSource.match(/const DESCENT_FIELDS = \[([\s\S]*?)\n\];/);
want(descBlock !== null, "the page's DESCENT_FIELDS table is where the gate expects it");
if (descBlock) {
  const fields = descBlock[1].split(/\n  \{/).filter((b) => /id: "/.test(b)).map((b) => ({
    block: b,
    id: (b.match(/id: "([^"]+)"/) || [, null])[1],
    // Sources are written across continuation lines, so the concatenated literal is
    // reassembled before it is parsed. A regex that read only the first fragment would
    // silently accept a source whose second half named nothing.
    source: [...b.matchAll(/source: "((?:[^"\\]|\\.)*)"|\+ "((?:[^"\\]|\\.)*)"/g)]
      .map((m) => m[1] ?? m[2]).join(""),
  }));
  want(fields.length >= 15, `the readouts card enumerates its rows (${fields.length} found)`);

  // ---- THE MARKUP CONTRACT, BOTH DIRECTIONS --------------------------------
  //
  // A row in the table with no element renders nowhere; an element with no table row shows
  // a dash forever and nothing says which. `put()` is deliberately tolerant of a missing
  // element so a removed panel cannot take the frame loop down, and this is what keeps that
  // tolerance from letting a whole card go dark in silence.
  const htmlDescIds = [...htmlSource.matchAll(/id="(desc-[A-Za-z0-9_-]+)"/g)].map((m) => m[1]);
  for (const f of fields) {
    want(htmlDescIds.includes(f.id), `readout row "${f.id}" has an element in index.html`,
      "a row the table declares and the markup does not carry renders nowhere at all");
  }
  for (const id of htmlDescIds) {
    want(fields.some((f) => f.id === id),
      `element "${id}" in the markup is declared in DESCENT_FIELDS`,
      "a row with no declared source is a digit with no provenance, which is the whole "
      + "thing WB-7 forbids");
  }

  // ---- EVERY SOURCE IS ONE OF THE FOUR FORMS, AND EVERY FORM RESOLVES ------
  const renderSrc = appSource.slice(appSource.indexOf("function renderDescent"));
  // COMMENTS STRIPPED for every check below that is about CODE. Two of these checks were
  // written against the raw text and one of them passed on a tooltip: "read back from
  // holon_atom_band_solve" satisfied a test meant to establish that the door is CALLED.
  // A checker that cannot tell a call from a sentence about a call is a checker that goes
  // green on the defect it names.
  const renderCode = renderSrc.replace(/^\s*\/\/.*$/gm, "");
  const solveFn = appSource.match(/function maybeSolveAtomBand\(w, atom\) \{\n([\s\S]*?)\n\}/);
  const palette = JSON.parse(readFileSync(join(here, "species_palette.json"), "utf8"));
  const paletteHas = (f) => palette.species.some((s) => s[f] !== undefined);

  for (const f of fields) {
    const [kind, rest] = [f.source.slice(0, f.source.indexOf(":")), f.source.slice(f.source.indexOf(":") + 1)];
    if (!["live", "export", "declared", "computed"].includes(kind)) {
      no(`readout row "${f.id}" declares an unknown source kind`,
        `source reads "${f.source}" — the four forms are live:, export:, declared: and `
        + "computed:, and a fifth would be a digit whose provenance nothing checks");
      continue;
    }
    if (kind === "live") {
      const names = rest.split(",").map((s) => s.trim()).filter(Boolean);
      const stray = names.filter((n) => !required.includes(n));
      want(names.length > 0 && stray.length === 0,
        `readout row "${f.id}" traces to REQUIRED_EXPORTS (${names.length})`,
        stray.length ? `not required: ${stray.join(", ")} — a live row must trace to an `
          + "export the boot refuses to run without" : "no export named");
    } else if (kind === "export") {
      const name = rest.trim();
      want(pending.includes(name),
        `readout row "${f.id}" waits on a PENDING_EXPORTS name (${name})`,
        "a row fenced on an export that is on neither list is fenced on nothing");
      // AND IT IS WIRED TO THAT NAME. The failure this catches is real and silent: pairing
      // the spin row with the charge-radius export renders a length where a spin belongs,
      // and every other check on this page would stay green.
      const at = renderCode.indexOf(`"${f.id}"`);
      const near = at >= 0 ? renderCode.slice(at, at + 420) : "";
      // STRUCTURALLY, not by mention. The export's name must sit in `exportRow`'s first
      // argument beside this row, or be paired with the id in the table the loop walks —
      // a name that merely appears in the row's tooltip proves nothing about what the row
      // reads, and that is exactly how the solve check above went green on nothing.
      want(near.includes(`exportRow("${name}"`) || near.includes(`"${f.id}", "${name}"`),
        `readout row "${f.id}" is wired to ${name} in renderDescent`,
        at < 0 ? "the row is never rendered at all"
          : `the render near "${f.id}" does not pass ${name} to exportRow; a row wired to a `
            + "different export shows the wrong quantity under the right label");
    } else if (kind === "declared") {
      const [file, fld] = rest.split("#");
      want(!!file && !!fld, `readout row "${f.id}" names a file and a field`, f.source);
      if (file === "species_palette.json") {
        want(paletteHas(fld),
          `readout row "${f.id}" cites a field the committed species table carries (${fld})`,
          `species_palette.json has no "${fld}" on any species — a DECLARED number must come `
          + "from a table that exists, or it is not declared, it is invented");
      } else {
        // A declared source whose file is not in the tree yet is legitimate — that is what
        // pending means — but the page must then FENCE the row rather than render it. The
        // guard is checked in the source, because the file's absence is the state today.
        let present = true;
        try { readFileSync(join(here, file)); } catch { present = false; }
        if (present) ok(`readout row "${f.id}" cites ${file}, which is in the tree`);
        else want(/State\.lawProbe/.test(renderSrc),
          `readout row "${f.id}" fences while ${file} is absent`,
          "the file the row declares is not in the tree and the render does not guard on "
          + "its absence — the row would draw whatever `undefined` formats to");
      }
    } else {
      const inputs = rest.split(",").map((s) => s.trim()).filter(Boolean);
      want(inputs.length >= 2, `computed row "${f.id}" names its inputs (${inputs.length})`,
        "arithmetic with one named input has an unnamed one, and an unnamed input is how a "
        + "constant walks into a readout");
      for (const inp of inputs) {
        const holon = inp.match(/holon_[a-z0-9_]+/);
        if (holon) {
          want(declared.has(holon[0]), `computed row "${f.id}" input ${holon[0]} is declared`);
          continue;
        }
        const other = inp.match(/^desc-[A-Za-z0-9_-]+$/);
        if (other) {
          want(fields.some((g) => g.id === other[0]),
            `computed row "${f.id}" input ${other[0]} is another declared row`);
        }
        const filefield = inp.match(/^([A-Za-z0-9_.-]+\.json)#([A-Za-z0-9_]+)$/);
        if (filefield) {
          want(filefield[1] !== "species_palette.json" || paletteHas(filefield[2]),
            `computed row "${f.id}" input ${inp} resolves in the committed table`);
        }
      }
    }
  }

  // ---- THE DECLARED DOORS' SENTINELS -------------------------------------
  //
  // A door that cannot serve a DECLARED value returns a sentinel rather than a plausible
  // number (`u32::MAX`, `0.0` — nucleus.rs's own header). Two ways to get this wrong, and
  // both put a false measurement on screen, so both are planted here against the page's own
  // readers rather than against a copy of them.
  //
  //   * `u32::MAX` crosses the i32 ABI as **-1**. A guard spelled `=== 4294967295` is false
  //     for exactly the value it exists to catch, and the row renders "I = -1/2".
  //   * ZERO IS A REAL SPIN. ¹⁶O and ¹²C both have spin 0, so the falsiness guard this would
  //     ordinarily be written as fences the true value for two of the ten elements the page
  //     can draw.
  const u32Src = appSource.match(/function declaredU32\(v\) \{\n([\s\S]*?)\n\}/);
  const posSrc = appSource.match(/function declaredPositive\(v\) \{\n([\s\S]*?)\n\}/);
  want(u32Src !== null && posSrc !== null, "the page implements both sentinel readers");
  if (u32Src && posSrc) {
    const dU32 = new Function("v", u32Src[1]);
    const dPos = new Function("v", posSrc[1]);
    want(dU32(-1) === null, "the u32 sentinel is caught as it actually arrives (-1)",
      `got ${dU32(-1)} — a wasm u32 reaches JavaScript through the i32 ABI, so u32::MAX is `
      + "-1 here and a guard written against 4294967295 alone never fires");
    want(dU32(4294967295) === null, "and as its unsigned spelling too");
    want(dU32(0) === 0, "spin 0 is a VALUE, not an absence",
      "¹⁶O has spin 0; a falsy test would fence the true value for two of the ten elements "
      + "this page can draw");
    want(dU32(1) === 1 && dU32(3) === 3, "ordinary spins pass through");
    want(dPos(0) === null && dPos(0.0) === null,
      "the real-valued sentinel 0.0 is fenced — a zero charge radius is a point nucleus");
    want(dPos(-1) === null && dPos(NaN) === null,
      "a negative or non-finite reading is fenced rather than rendered",
      "a door that returned one would be broken, and drawing it puts the breakage on screen "
      + "as a measurement");
    want(dPos(0.8414) === 0.8414, "a real charge radius passes through");

    // AND AGAINST THE SHIPPED ARTIFACT, so the readers are tested on the doors' real output
    // rather than on what this file believes the doors return.
    if (typeof w.holon_nucleus_spin2 === "function") {
      want(dU32(w.holon_nucleus_spin2(11)) === null
        && dPos(w.holon_nucleus_charge_radius_fm(11)) === null,
        "an element with no declared nucleus fences on the page's own readers (Z = 11)",
        `the doors returned spin2 ${w.holon_nucleus_spin2(11)} and radius `
        + `${w.holon_nucleus_charge_radius_fm(11)}, which the page did not fence`);
      want(dU32(w.holon_nucleus_spin2(8)) === 0 && dPos(w.holon_nucleus_charge_radius_fm(8)) > 0,
        "and ¹⁶O's declared spin of 0 survives the same readers",
        "the sentinel test must not swallow a real zero");
      // THE THREE DECLARED ROWS ARE INDEPENDENT, measured rather than assumed: the mass
      // table covers more elements than the nucleus table, so a single "is it declared"
      // flag would fence a mass the engine serves or show a spin it does not.
      want(dPos(w.holon_nucleus_mass_u(11)) > 0,
        "mass is declared where spin and charge radius are not (Z = 11), so the rows fence "
        + "independently",
        "if this ever changes the page may share one flag across the three rows; today it "
        + "must not");
    }
    // Each row must go through the reader rather than formatting the raw return.
    // The window looks BOTH WAYS. One of these three computes its value before it paints,
    // and a forward-only window reported it as unguarded when it was guarded — a false
    // failure is a check people learn to route around, which is worse than none.
    for (const [id, reader] of [["desc-nuc-spin", "declaredU32"],
      ["desc-nuc-radius", "declaredPositive"], ["desc-nuc-mass", "declaredPositive"]]) {
      const at = renderCode.indexOf(`"${id}"`);
      const near = at >= 0
        ? renderCode.slice(Math.max(0, at - 420), at + 420) : "";
      want(near.includes(`${reader}(`),
        `readout row "${id}" passes its door through ${reader}`,
        at < 0 ? "the row is never rendered at all"
          : "formatting the raw return renders the sentinel as a measurement — -1 for a "
            + "spin, or a zero charge radius as a point nucleus");
    }
  }

  // ---- AND WHETHER THAT FENCE CAN FIRE AT ALL (WB-2.4b's rule) ------------
  //
  // "An instrument that cannot fire is worse than none" — the FSD says it of the periodic
  // box's gravity refusal, and it applies here: this page draws from a ten-element palette,
  // and if every one of those has a declared nucleus then the sentinel fence is CORRECT and
  // UNREACHABLE from the page's own species set. That is worth saying on the page rather
  // than letting the row look like a live guard, and it is worth checking in both
  // directions — if the palette grows past the nucleus table, the sentence must change.
  if (typeof w.holon_nucleus_spin2 === "function") {
    const paletteZ = palette.species.map((s) => s.Z);
    const undeclared = paletteZ.filter((z) => (w.holon_nucleus_spin2(z) >>> 0) === 0xFFFFFFFF
      || !(w.holon_nucleus_charge_radius_fm(z) > 0));
    const htmlFlat = htmlSource.replace(/<!--[\s\S]*?-->/g, "").replace(/\s+/g, " ");
    const saysUnreachable = /not reachable from this page's own species set/i.test(htmlFlat);
    want(undeclared.length === 0 ? saysUnreachable : !saysUnreachable,
      undeclared.length === 0
        ? `the page states that the sentinel fence is unreachable from its ${paletteZ.length}-element palette`
        : `the page does not claim an unreachable fence — Z ${undeclared.join(", ")} have no declared nucleus`,
      undeclared.length === 0
        ? "every species this page can draw has a declared nucleus, so the fence cannot fire "
          + "here. WB-2.4b's rule: a shell must not advertise an unfireable refusal as a live "
          + "fence, so the page has to say which it is"
        : `Z ${undeclared.join(", ")} now fence, so the page's 'unreachable' sentence is `
          + "false and must be removed");
  }

  // ---- WB-5.2: THE ATOM BAND IS NEVER SILENTLY ZEROED ----------------------
  //
  // `holon_atom_band_energy` returns exactly 0.0 for an atom whose solve was never kept, and
  // its companion `holon_atom_band_exit` returns 4 — "not computed". Zero hartree is a
  // number a reader takes for an energy, so the page must consult the exit before painting
  // the value. This is the artifact's own refusal being displayed as a refusal, which is
  // exactly what WB-5.2 asks: never faked, never interpolated across, never silently zeroed.
  //
  // The engine's contract is the reason this is checkable at all: the getters are read-backs
  // of the LAST solve, so a page that never calls the door reads four zeros that look fine.
  want(/\bmaybeSolveAtomBand\(w,/.test(renderCode)
    && solveFn !== null && /w\.holon_atom_band_solve\(/.test(solveFn[1]),
    "the page RUNS the atom band's solve rather than only reading its getters",
    "the four getters return the last kept solve — zeros and exit 4 if there is none — so a "
    + "page that never opens holon_atom_band_solve displays a solve that never happened. "
    + "The call is what is checked, not a mention of it: this test passed for an hour on a "
    + "tooltip that said 'read back from holon_atom_band_solve'");
  want(/w\.holon_atom_band_exit\(/.test(renderCode) && /!\s*solved\b/.test(renderCode),
    "the atom band's value rows are guarded by the engine's own exit code",
    "WB-5.2: the energy row must consult holon_atom_band_exit before printing, or an "
    + "unsolved atom reads +0.000000 Ha and nothing on the page says it was never computed");
  // AND THE SOLVE IS NOT RUN PER FRAME. The engine's own door says a molecule's FCI is
  // milliseconds; a per-frame solve would spend the scene's budget on a readout and WB-6.2
  // would pay for it in dilated time. A throttle is the property, so a throttle is checked.
  want(solveFn !== null && /BAND_SOLVE_INTERVAL_MS|atMs/.test(solveFn[1]),
    "the atom band's solve is throttled, not run every frame",
    "renderDescent runs on every frame; an unthrottled millisecond solve inside it is a "
    + "readout charging the scene its whole time budget");

  // ---- WB-1.6: MEMBERSHIP COMES FROM THE CENSUS, NEVER FROM A DISTANCE -----
  //
  // §11.2 forbids this by name — "never from a distance heuristic in JavaScript" — and the
  // page has every coordinate it would need to break the rule, which is exactly why the
  // ban is worth mechanising. The pair doors are the ones a heuristic would reach for.
  const FORBIDDEN_IN_DESCENT = ["holon_pair_bonded", "holon_pair_r", "holon_pair_i", "holon_pair_j"];
  const reached = FORBIDDEN_IN_DESCENT.filter((n) => renderCode.includes(n));
  want(reached.length === 0,
    "the descent reads membership from the census export, not from a separation",
    reached.length ? `renderDescent reaches for ${reached.join(", ")} — WB-1.6 requires the `
      + "census's own verdict, and a distance in this file is not the census's bond criterion"
      : undefined);
}

// ------------------- 6c5. a PENDING row cannot evaluate, let alone display, a digit
//
// `exportRow` is the whole of WB-7 for the readouts card: every export-served row goes
// through it, and it takes its value as a THUNK so an absent export cannot produce a
// number. That is a property, so it is planted rather than asserted — the thunk THROWS,
// and if the guard is ever inverted this gate reports the throw instead of a wrong string.
const rowSrc = appSource.match(
  /function exportRow\(name, has, kind, value, traceLive, tracePending\) \{\n([\s\S]*?)\n\}/);
want(rowSrc !== null, "the page implements exportRow");
if (rowSrc) {
  let row;
  try {
    row = new Function("name", "has", "kind", "value", "traceLive", "tracePending", rowSrc[1]);
  } catch (e) { no("exportRow's body could not be reconstructed for testing", String(e)); }
  if (row) {
    const boom = () => { throw new Error("a pending row evaluated its value"); };
    let absent = null, threw = null;
    try {
      absent = row("holon_nucleus_spin2", () => false, "declared", boom, "t", "p");
    } catch (e) { threw = String(e.message); }
    want(threw === null,
      "an absent export does not even EVALUATE the row's value",
      `the value thunk ran: ${threw} — a guard that computes first and decides after is one `
      + "refactor away from displaying what it computed");
    want(absent && absent.kind === "pending" && absent.text.includes("holon_nucleus_spin2"),
      "an absent export renders PENDING and names the export",
      `got ${JSON.stringify(absent)}`);
    const shown = row("holon_nucleus_spin2", () => true, "declared", () => "I = 5/2", "t", "p");
    want(shown.kind === "declared" && shown.text === "I = 5/2" && shown.trace === "t",
      "a present export renders its value under the tag the row declares",
      `got ${JSON.stringify(shown)}`);
    // THE TAG IS THE ROW'S, NOT THE MECHANISM'S. A measured input read back through an
    // export is still DECLARED (WB-1.7); if exportRow ever hard-coded "live" the nucleus
    // rows would start claiming the engine computes a charge radius.
    want(shown.kind !== "live",
      "an export-served DECLARED row is not relabelled LIVE by having been served",
      "WB-1.7: a function returning a measured input does not make the input computed");
    // A PRESENT DOOR SERVING ITS SENTINEL IS STILL A FENCE, and it must carry the fence's
    // TAG and not just its wording. This row read "DECLARED · FENCED — no charge radius is
    // declared" until the sentinel path was walked end to end against a sodium atom: the
    // text was right and the tag contradicted it, and the tag is what the eye sorts by.
    const sentinel = row("holon_nucleus_spin2", () => true, "declared", () => null, "t", "p");
    want(sentinel.kind === "pending",
      "a door that serves its sentinel fences with the FENCE's tag, not the row's",
      `got kind "${sentinel.kind}" — a row tagged DECLARED whose text reads FENCED is a `
      + "panel contradicting itself, and the tag is the half a reader sorts by");
    want(!/\d/.test(sentinel.text.replace(/holon_[a-z0-9_]+/g, "")),
      "and it draws no digits of its own",
      `text was "${sentinel.text}"`);
  }
}

// ------------------------------- 6c6. the bit-identity gate is bits, not a tolerance
//
// §11.1: one determinant kernel runs every space, so wasm and native agree BITWISE rather
// than closely — which is the whole reason this row can be a gate. A tolerance here would
// pass on two numbers that differ and would throw away the property the lane kernel exists
// to make checkable.
const verdictSrc = appSource.match(
  /function lawProbeVerdict\(wasmBits, nativeBits\) \{\n([\s\S]*?)\n\}/);
want(verdictSrc !== null, "the page implements lawProbeVerdict");
if (verdictSrc) {
  const body = verdictSrc[1];
  want(!/Math\.abs|<\s*1e-|epsilon|tolerance/i.test(body),
    "the bit-identity comparison carries no tolerance",
    "a near-equality in this row would report EQUAL TO THE BIT for two different numbers");
  let verdict;
  try { verdict = new Function("wasmBits", "nativeBits", body); }
  catch (e) { no("lawProbeVerdict could not be reconstructed for testing", String(e)); }
  if (verdict) {
    want(verdict("3ff0000000000000", "3FF0000000000000") === true,
      "identical patterns compare EQUAL regardless of hex case");
    want(verdict("3ff0000000000000", "3ff0000000000001") === false,
      "a one-ulp difference is a MISMATCH, not a pass",
      "the last hex digit is one unit in the last place — the smallest disagreement two "
      + "doubles can have, and the one a tolerance would hide");
    want(verdict(null, "3ff0000000000000") === null && verdict("3ff0000000000000", null) === null,
      "a missing half is PENDING, never a pass",
      "a comparison with nothing that reported agreement would be the vacuous-success shape "
      + "with a checkmark on it");
  }
}

// The pinned native value itself, when the engine lane has written it. Absent today, and
// the check is written so that its arrival starts checking rather than needing an edit.
let lawProbe = null;
try { lawProbe = JSON.parse(readFileSync(join(here, "law_probe.json"), "utf8")); } catch { /* below */ }
if (lawProbe === null) {
  ok("law_probe.json is not in the tree yet — the bit-identity row PENDS, and this gate "
    + "starts comparing the moment the engine lane writes it");
} else {
  want(typeof lawProbe.probe === "string" && typeof lawProbe.pinned_by === "string"
    && typeof lawProbe.energy === "number"
    && /^[0-9a-fA-F]{16}$/.test(lawProbe.energy_bits_hex || ""),
    "law_probe.json carries a probe, its bits, its value and the test that pins it",
    `got ${JSON.stringify(lawProbe)}`);
  // The file's own two halves must agree before it is used to judge anything else.
  const dv = new DataView(new ArrayBuffer(8));
  dv.setFloat64(0, lawProbe.energy, false);
  const fileBits = [...new Uint8Array(dv.buffer)]
    .map((b) => b.toString(16).padStart(2, "0")).join("");
  want(fileBits === String(lawProbe.energy_bits_hex).toLowerCase(),
    "law_probe.json's hex bits are the bits of its own energy value",
    `energy ${lawProbe.energy} has bits ${fileBits}, file says ${lawProbe.energy_bits_hex}`);
  if (typeof w.holon_law_probe === "function") {
    dv.setFloat64(0, w.holon_law_probe(), false);
    const wasmBits = [...new Uint8Array(dv.buffer)]
      .map((b) => b.toString(16).padStart(2, "0")).join("");
    want(wasmBits === String(lawProbe.energy_bits_hex).toLowerCase(),
      "the shipped wasm's law probe equals the pinned native value TO THE BIT",
      `wasm ${wasmBits} vs native ${lawProbe.energy_bits_hex} — §11.1's claim is that one `
      + "determinant kernel runs every space, so these are the same arithmetic or the claim "
      + "is wrong");
  } else {
    ok("law_probe.json is pinned but this artifact has no holon_law_probe — the row PENDS "
      + "on the wasm rather than on the referee");
  }
}

// --------------------------------- 6c3. EVERY fence the page displays is registered
//
// R3 of the retirement battery, taken to FULL at the lead's ruling: partial was not it.
// The RECORD figures and the band cites were gated; the page's runtime fences and its
// not-served list were prose, and prose is where a fence goes to rot — the F-2 incident
// was exactly that, one file over.
//
// Three properties per fence, and the third is the one the fence law actually demands:
// an OWNER (a fence nobody owns is suppression, by that ledger's own words), an EXIT
// (a fence with no exit is architecture, which the law forbids), and a REGISTER ROW that
// EXISTS in FENCES.md. Cited by row ID, not by line: an id survives the register being
// reordered and a line number does not.
/// Does this line carry a POSITIVE certification verdict?
///
/// NEGATION-AWARE, and it was not. The matcher was `/CERTIFIED/.test(line)`, and
/// RUNG2_RESULTS.md line 14 reads "THE FLUID-ELEMENT TIER IS NOT CERTIFIED" — on which
/// that test returns TRUE. Rung 2 is node G, so a band wired to cite its results doc
/// would have satisfied both halves of the flip test and the gate would have demanded,
/// in the voice of a rule, exactly the flip §9c forbids. A substring match cannot tell a
/// verdict from its negation, and the negation is the common case for a lane that banked
/// branch (d).
///
/// So: find each CERTIFIED token and look at the words immediately before it. A token
/// with a negator in front of it is not a verdict; a line counts only if some occurrence
/// is un-negated.
function carriesPositiveVerdict(line) {
  const re = /\bCERTIFIED(?:-[A-Z]+)?\b/gi;
  let m;
  while ((m = re.exec(line)) !== null) {
    const before = line.slice(Math.max(0, m.index - 24), m.index);
    if (!/\b(not|never|non|un|no)\b[\s\-]*$/i.test(before)) return true;
  }
  return false;
}

const fenceRegisterSrc = appSource.match(/const FENCE_REGISTER = \{([\s\S]*?)\n\};/);
const notServedSrc = appSource.match(/const NOT_SERVED = \[([\s\S]*?)\n\];/);
want(fenceRegisterSrc !== null && notServedSrc !== null,
  "the page's fence register and not-served list are where the gate expects them");

if (fenceRegisterSrc && notServedSrc) {
  const ledger = readFileSync(join(repoRoot, "FENCES.md"), "utf8");
  // A row id counts as existing only where the ledger uses it AS A ROW — leading pipe,
  // the id, a pipe. "P13" appearing in a sentence is a mention, not a registration, and a
  // check that accepted a mention would pass on a fence nobody had actually filed.
  const rowExists = (id) =>
    new RegExp(`^\\|\\s*\\*{0,2}${id}\\*{0,2}\\s*\\|`, "m").test(ledger);

  want(rowExists("P13") && !rowExists("P999"),
    "the row-existence test distinguishes a filed row from an absent one",
    "the test cannot tell a real row id from an invented one, so every check below is void");

  const entries = [
    ...[...fenceRegisterSrc[1].matchAll(/(\w+): \{([\s\S]*?)\n  \},/g)]
      .map((m) => ({ name: m[1], body: m[2], kind: "runtime fence" })),
    ...notServedSrc[1].split(/\n  \{/).filter((b) => /what:/.test(b))
      .map((b) => ({ name: (b.match(/what: "([^"]+)"/) || [, "?"])[1], body: b, kind: "not-served panel" })),
  ];
  want(entries.length >= 8, `every displayed fence is enumerable (${entries.length} found)`);

  for (const { name, body, kind } of entries) {
    const owner = /owner: "([^"]+)"/.exec(body);
    const exit = /exit: "((?:[^"\\]|\\.)*)"/.exec(body);
    const reg = /register: "([^"]+)"/.exec(body);
    want(!!owner, `${kind} "${name}" names an OWNER`,
      "a fence nobody owns is suppression, by the ledger's own law");
    want(!!exit && exit[1].length > 25, `${kind} "${name}" names a substantive EXIT`,
      "a fence with no exit is architecture, and the ledger forbids that word");
    want(!!reg && rowExists(reg[1]),
      `${kind} "${name}" cites a FENCES.md row that exists (${reg ? reg[1] : "none"})`,
      reg ? `row ${reg[1]} is not a row in FENCES.md` : "no register id — the page displays a fence the register has never seen");
  }
}

// ------------------------------------------- 6d. the PROSE cannot outlive the engine
//
// FENCES.md F-2 caught this page telling viewers "there is no barostat in this engine"
// months after the barostat landed. The absence LIST was already gated — that is what made
// me un-fence the gravity panel the day `holon_set_gravity` appeared — but the page's
// PROSE was not, so half the claim was mechanised and half was a sentence nobody re-read.
// The same header also still listed gravity among the absent, and gravity is mine.
//
// So: for each capability, the export that proves it exists and the phrase that would deny
// it. If the export resolves and the phrase is in the shipped text, this fails. It is the
// inverted check aimed at prose instead of at panels, and it closes the half of F-2 that
// the absence list never covered.
const PROSE_CLAIMS = [
  { export: "holon_box_scale", deny: /there is no barostat/i,
    say: "the barostat landed (holon_box_scale); only the SETPOINT door is fenced, by design" },
  { export: "holon_set_gravity", deny: /no gravitational (force )?term exists/i,
    say: "gravity landed (holon_set_gravity / holon_set_gravity_vec) and the panel is live" },
  { export: "holon_pressure", deny: /no pressure readout/i,
    say: "holon_pressure is the readout, with holon_pressure_defined honoured" },
];
const shippedText = readFileSync(join(here, "index.html"), "utf8") + appSource;
for (const claim of PROSE_CLAIMS) {
  const exists = typeof w[claim.export] === "function";
  const denied = claim.deny.test(shippedText);
  want(!(exists && denied),
    `the page does not deny a capability the engine has (${claim.export})`,
    `${claim.export} resolves, but the shipped text still matches ${claim.deny} — ${claim.say}`);
}

// ------------------------------- 6e. THE TWO-BOX LAW, made falsifiable on the page
//
// The law's whole content is that two knobs are never the same knob, and the way that goes
// wrong is measurable rather than aesthetic. An earlier draft of the spec said "the
// box-scale door IS the zoom"; implementing that literally makes zooming COMPRESS the
// water. So both directions are pinned here, with the wrong door planted as the mutation.

const zb = await freshEngine();
zb.holon_set_dims(1);
zb.holon_set_boundary(0);
zb.holon_table_generate(0.6, 12.0, 192);
zb.holon_set_calibration(2.0e6);
zb.holon_reset(24);
zb.holon_rebase();

const worldVolume = () => zb.holon_width() * zb.holon_height() * zb.holon_depth();
const worldDensity = () => zb.holon_atom_count() / worldVolume();

// (i) THE ZOOM IS A PAGE-SIDE RATIO AND CALLS NOTHING. Read out of the shipped source
// rather than asserted: the zoom handler must contain no engine call at all. A zoom that
// reached for `holon_box_scale` would be the wrong door, and it would look right on screen
// for exactly as long as nobody read the pressure.
const zoomHandler = appSource.match(/UI\["sheet-zoom"\]\?\.addEventListener\("input",([\s\S]*?)\n  \}\);/);
want(zoomHandler !== null, "the page has a zoom handler");
if (zoomHandler) {
  const calls = [...zoomHandler[1].matchAll(/\bw\.(holon_[a-z0-9_]+)|State\.w\.(holon_[a-z0-9_]+)/g)]
    .map((m) => m[1] || m[2]);
  want(calls.length === 0,
    "the zoom touches NO engine call — it changes a ratio and nothing else",
    `the zoom handler calls ${calls.join(", ")}; the zoom must not reach into the Sim, and `
    + "reaching for holon_box_scale is the wrong door the two-box law exists to separate");
}

// (ii) THE WRONG DOOR'S SIGNATURE, MEASURED, so the gate knows what it is refusing.
// Affine scale multiplies density by 1/f³. This is not a hypothetical: it is what the
// superseded spec sentence would have produced, and the number is the reason it was
// superseded.
const d0 = worldDensity();
for (let k = 0; k < 3; k++) zb.holon_box_scale(0.5);
const densityRatio = worldDensity() / d0;
want(Math.abs(densityRatio - 512) < 1,
  `the HAND's door is affine and compresses: 3 halvings multiply world density by ${densityRatio.toFixed(1)}× (1/f³ = 512)`,
  `measured ${densityRatio}; if this is no longer 512 the box-scale door has changed meaning and `
  + "the two-box separation needs re-deriving");
want(zb.holon_atom_count() === 24,
  "and it removes nothing — every holon scales with the container",
  "box_scale dropped atoms, which would make it a zoom after all");

// (iii) THE SCENE BOX IS A QUOTIENT, and the page computes it that way. The law's own
// example: the same zoom on two different worlds must give different scene boxes.
const sceneBoxSrc = appSource.match(/function sceneBox\(w\) \{\n([\s\S]*?)\n\}/);
want(sceneBoxSrc !== null && /holon_width\(\) \/ \(2 \* z\)/.test(sceneBoxSrc[1]),
  "the scene box is world ÷ zoom, so the same zoom on two worlds gives two scene boxes",
  "sceneBox does not divide the world extent by the zoom — a scene box that is a fixed "
  + "LENGTH would decouple the view from the hand, which the law forbids in the other direction");

// (iv) WHOLE-ONLY OBSERVABLES COME FROM THE WHOLE BOX. The hazard the cut creates: a view
// filter that computed "the atoms I am drawing" and reported THEIR temperature would look
// plausible at every zoom and be wrong at all but one. The page must read the engine's
// whole-box readouts, never the drawn subset.
const WHOLE_ONLY = ["holon_temperature", "holon_pressure", "holon_census_molecules"];
for (const sym of WHOLE_ONLY) {
  const used = new RegExp(`w\\.${sym}\\(\\)`).test(appSource);
  want(used, `the page reads ${sym} from the engine (whole box), not from the drawn subset`,
    "a whole-only observable computed over scene members is the two-box law violated in "
    + "the direction no screenshot would reveal");
}
// And the membership predicate must not be reachable from those readouts: the scene cut
// lives in the renderer, and `sceneMembers` is called from exactly one place.
const memberCalls = (appSource.match(/sceneMembers\(/g) || []).length;
want(memberCalls === 2,
  `the scene-membership cut has exactly one call site plus its definition (found ${memberCalls})`,
  "more call sites means membership is leaking out of the renderer, which is how a "
  + "whole-only number starts being computed over a fraction");

// ------------------------------- 6f. the view centre is the observer's, and it clamps
//
// The lead's ruling, derived from the FSD: the centre is AIM and aim belongs to the
// observer, so the camera target owns it. Two properties, and the second is what keeps a
// zoomed view from showing a region that is not in the domain at all.
const sceneBoxFn = appSource.match(/function sceneBox\(w\) \{\n([\s\S]*?)\n\}/);
want(sceneBoxFn !== null, "sceneBox is where the gate expects it");
if (sceneBoxFn) {
  const body = sceneBoxFn[1];
  want(/State\.camera\.target/.test(body),
    "the scene box is centred on the CAMERA TARGET, not hard-pinned to the world centre",
    "the centre is aim, and aim is the observer's axis — pinning it to the world centre "
    + "makes deep zoom on a shell-opener scene able to look only at vacuum");
  want(/clamp/.test(body),
    "the scene box is CLAMPED inside the world box — near a wall it slides, never protrudes",
    "an unclamped scene box aimed near a face shows a region outside the domain, which is "
    + "not vacuum but nothing at all");
  // AIM MUST NOT TOUCH THE PHYSICS. The whole point of separating the axes is that the
  // observer's knob reaches no engine call — the same property the zoom handler is gated
  // on, checked at the other end of the same law.
  const aim = appSource.match(/canvas\.addEventListener\("dblclick",([\s\S]*?)\n  \}\);/);
  want(aim !== null, "the page has an aim control");
  if (aim) {
    const calls = [...aim[1].matchAll(/\bw\.(holon_[a-z0-9_]+)|State\.w\.(holon_[a-z0-9_]+)/g)]
      .map((m) => m[1] || m[2]);
    want(calls.length === 0,
      "aiming touches NO engine call — the observer's axis, never the hand's",
      `the aim handler calls ${calls.join(", ")}; aiming has no physical meaning and must `
      + "not reach the Sim, or the two axes the two-box law separates are coupled again");
  }
}

// ------------------------- 6g. a fence is a BUG UNDER REPAIR, never content
//
// Operator's law (FSD, cd2160e): any "content" saying "we refuse, and the honesty is the
// point" is not content, it is a bug. Honesty is unchanged — every citation gate stands and
// no band flips without its certificate — but the STORY a fence tells is its DEBT, its
// OWNER and THE BUILD PAYING IT, in the present tense.
//
// This gate is a wording check, which is a weaker instrument than the rest of this file,
// and it is scoped to what a wording check can actually establish: the phrases that make
// refusal the point. It cannot tell a well-written debt from a badly written one. Stated
// so that "6g green" is not read as "the tense is right everywhere".
const REFUSAL_AS_POINT = [
  /\bthe honesty is the point\b/i,
  /\brefusing is\b[^.]{0,40}\b(the point|honest|a feature)\b/i,
  /\bwe refuse\b[^.]{0,30}\band that is\b/i,
  /\bfence is honest content\b/i,
];
// WHITESPACE IS NORMALISED FIRST, and that is not tidiness — it is the difference between
// this check working and this check having been decorative. index.html carried the sentence
// "a fence is honest content on it" for as long as the check existed, and the check passed:
// the phrase was broken across a wrapped line, so `\bfence is honest content\b` never
// matched the newline and the indentation between "honest" and "content". A text gate that
// only sees phrases the author happened not to wrap is a gate whose coverage depends on the
// width of the editor.
const pageText = (readFileSync(join(here, "index.html"), "utf8").replace(/<!--[\s\S]*?-->/g, "")
  + appSource.replace(/\/\*[\s\S]*?\*\//g, "").replace(/^\s*\/\/.*$/gm, ""))
  .replace(/\s+/g, " ");
for (const pat of REFUSAL_AS_POINT) {
  want(!pat.test(pageText),
    `no refusal-as-the-point wording matching ${pat}`,
    "a fence states its debt, its owner and the build paying it — present tense. The "
    + "operator's law: refusal presented as the point is a bug wearing content's clothes");
}

// ---------------------------------------------------------------- 7. the inverted check

// --------------------------------------------- 5c. the hand on the box (WB-2.2), now served
//
// This block exists because holon_box_scale's entry was DELETED from
// FENCE_JUSTIFYING_ABSENCES below. The control IS the box: the page compresses the
// world and reads the pressure back; the move's cost is posted to the ledger's hand
// column, so the energy gate stays a gate through it.

const press = await freshEngine();
press.holon_set_dims(1);
press.holon_set_boundary(0);
press.holon_table_generate(0.6, 12.0, 192);
press.holon_reset(12);

want(press.holon_pressure_defined() === 0,
  "under walls the virial is not the pressure, and the engine says so");
const w0 = press.holon_width();
want(press.holon_box_scale(0.9) === 1, "a modest compression is accepted");
want(Math.abs(press.holon_width() - 0.9 * w0) < 1e-9 * w0, "the box actually scales");
want(press.holon_box_scale(0.0) > 1, "a zero factor is refused by name");
want(press.holon_box_scale(1e-9) > 1, "collapsing the box below the wall inset is refused");

press.holon_rebase();
for (let f = 0; f < 50; f++) press.holon_step_frame(64);
press.holon_box_scale(0.95);
for (let f = 0; f < 50; f++) press.holon_step_frame(64);
want(press.holon_energy_gate() === 1,
  `the energy gate closes across a mid-run compression (drift ${press.holon_drift().toExponential(2)} vs bound ${press.holon_drift_bound().toExponential(2)} Ha) — an unledgered scale opens it by the move's cost`);

// The boundary door is browser-reachable and refuses BY NAME: the compressed box's half
// edge is now under the force law's reach, so wrapping it would break the minimum image.
// 101 == BOUNDARY_REFUSED (100) + BreaksPeriodicImages (1); the page names this code.
const wrapRefused = press.holon_set_boundary(2);
want(wrapRefused === 101 && press.holon_pressure_defined() === 0,
  `wrapping the compressed box is refused by the boundary door (code ${wrapRefused}) and the readout stays undefined`,
  `expected 101 (BOUNDARY_REFUSED + BreaksPeriodicImages), got ${wrapRefused}; pressure_defined ${press.holon_pressure_defined()}`);
// Widen it past the reach and the same door admits the wrap. The widening is COMPUTED from
// the door's own two numbers rather than guessed: the door requires reach <= half the
// shortest edge, and it is right to be strict, so the smoke asks it what it needs.
const reach = press.holon_legality_radius();
const halfEdge = press.holon_half_min_edge();
want(reach > halfEdge, `the refusal's numbers agree with it (reach ${reach.toFixed(3)} > half edge ${halfEdge.toFixed(3)} bohr)`,
  `the door refused but its own inequality does not hold: reach ${reach}, half edge ${halfEdge}`);
const widen = 1.01 * reach / halfEdge;
want(press.holon_box_scale(widen) === 1, `the box is widened by ${widen.toFixed(3)} so its half edge clears the reach`);
want(press.holon_set_boundary(2) === 0 && press.holon_pressure_defined() === 1,
  `on a legal periodic box (half edge ${press.holon_half_min_edge().toFixed(3)} >= reach ${press.holon_legality_radius().toFixed(3)} bohr) the readout IS a pressure, and boundary mode 2 reaches it`);
want(Number.isFinite(press.holon_pressure()), "and it reads");

// Exports whose ABSENCE is the stated reason a panel is fenced. If one appears, the fence
// text on the page has become false and this gate says so. The failure message is an
// instruction, not a complaint.
const FENCE_JUSTIFYING_ABSENCES = {
  holon_set_pressure: "no setpoint door ships: WB-2.2's control IS the box (holon_box_scale); pressure is the readout, not a target",
  holon_phase_call: "the blind classifier (WB-5.5) is fenced on the page because none exists",
  holon_q_tet: "the order parameters (WB-5.5) are fenced on the page because none are computed",
  holon_refinement_active: "local refinement (WB-1.2) is fenced on the page because none exists",
};
const appeared = Object.keys(FENCE_JUSTIFYING_ABSENCES).filter((n) => typeof w[n] === "function");
want(appeared.length === 0,
  "every fence on the page is still justified by a real absence in the engine",
  appeared.length
    ? appeared.map((n) => `${n} NOW EXISTS — ${FENCE_JUSTIFYING_ABSENCES[n]}. `
      + "Un-fence that panel and wire it, then delete this entry.").join("\n       ")
    : undefined);

// The tag discipline, checked in the shipped text rather than promised in a comment. The
// mock's own law (WB-7.1) allows a SYNTHETIC label; this page claims something stronger —
// that no displayed quantity is synthesized at all — and a claim in a header that nothing
// checks is the shape this repository keeps catching.
// Comments are stripped first, in both files. The claim is about what the page RENDERS and
// what the code COMPUTES — the header of each file discusses SYNTHETIC at length in order
// to say the word does not appear in the output, and a check that cannot tell the two
// apart would forbid explaining itself.
const html = readFileSync(join(here, "index.html"), "utf8").replace(/<!--[\s\S]*?-->/g, "");
const js = appSource.replace(/\/\*[\s\S]*?\*\//g, "").replace(/^\s*\/\/.*$/gm, "");
const offenders = [];
if (/SYNTHETIC/i.test(html)) offenders.push("a SYNTHETIC label in the rendered page");
if (/Math\.random/.test(js)) offenders.push("Math.random in app.js");
if (/sin\(\s*performance\.now/.test(js)) offenders.push("sin(performance.now()) in app.js");
want(offenders.length === 0,
  "no SYNTHETIC tag and no synthesized telemetry survive in the shipped page",
  offenders.join("; "));

// ------------------------------------------- N. the entry-point contract, both directions
//
// This section is about the OTHER page, and it belongs here because this lane owns both
// sides of the contract and this is the tree's only JS gate.
//
// Route B made `main` empty on wasm. That is what lets ONE artifact carry two apps —
// atoms3d's, which owns its world, and the workbench's, which owns no `Sim` at all and is
// fed across the bridge — because an app started from inside `init()` is an app no page
// can choose. The cost is that starting is now the page's job, and a page that forgets is
// a page that loads a 40 MB module, shows a canvas, and draws nothing on it forever.
//
// TWO DIRECTIONS, and neither alone is worth much:
//   * here: the page CALLS the entry point by name.
//   * `build-web.sh`: the finished artifact EXPORTS it, checked with
//     `WebAssembly.Module.exports` after wasm-opt, on the bytes the browser receives.
// A page calling a name nothing exports and an artifact exporting a name nobody calls are
// the same silence, and each check sees only one of them.
const atoms3dPage = readFileSync(join(repoRoot, "docs", "atoms3d", "index.html"), "utf8")
  .replace(/<!--[\s\S]*?-->/g, "");
want(/wasm\.holon3d_run_owned\s*\(/.test(atoms3dPage),
  "atoms3d names the app it wants — `main` no longer starts one",
  "the page must call wasm.holon3d_run_owned() after init(); without it the canvas stays black");
want(/typeof\s+wasm\.holon3d_run_owned\s*!==\s*["']function["']/.test(atoms3dPage),
  "and it checks the export is there before calling it, so a stale artifact fails loudly",
  "a page newer than its artifact would otherwise hang on a canvas nothing draws to");
const buildScript = readFileSync(
  join(repoRoot, "engine", "crates", "holon-render-3d", "build-web.sh"), "utf8");
want(/check-exports\.js/.test(buildScript),
  "and the build checks the artifact exports it, so the other direction is covered too",
  "build-web.sh must run check-exports.js on the finished wasm");

// HOW MANY CHECKS RAN, not just how many failed.
//
// This file threw once while being written — a botched `new Function` in the acuity block
// — and node printed a stack trace and exited. The exit code was non-zero, so CI would
// have caught SOMETHING, but every check after the throw silently did not run and the
// output named no failing property at all. A gate that can die halfway and report a stack
// trace is a gate whose coverage is unknown exactly when you most need it.
//
// So the count is asserted. The floor is deliberately well below the current total: its
// job is to catch a gate that stopped early, not to make adding a check a two-file edit.
// If it ever fires, the message says what to do — the number moves DOWN only when checks
// are deliberately removed.
const ran = passes + failures;
if (ran < MIN_CHECKS) {
  failures += 1;
  console.log(`  FAIL only ${ran} checks ran, expected at least ${MIN_CHECKS}`);
  console.log("       the run stopped early — look for a throw above, not a failing property");
}

reachedEnd = true;
console.log(failures === 0
  ? `\nworkbench artifact: all ${ran} checks passed`
  : `\nworkbench artifact: ${failures} of ${ran} check(s) failed`);
process.exit(failures === 0 ? 0 : 1);
