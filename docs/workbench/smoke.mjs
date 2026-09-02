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
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const AU_TO_FS = 0.024188843265857;

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

// Every `holon_*` the page actually CALLS, discovered from the source. The declared list
// above is a promise; this is the check that the promise covers the calls. `?.(` forms are
// deliberately included: an optional call on an export that does not exist is the false
// guard the atom viewer shipped for months.
const called = new Set([...appSource.matchAll(/\bw\.(holon_[a-z0-9_]+)/g)].map((m) => m[1]));
const undeclared = [...called].filter((n) => !required.includes(n)).sort();

// The other half of the same contract: every element id the page WRITES to must exist in
// the markup. `put()` and `tag()` are deliberately tolerant of a missing element so that a
// removed panel cannot take the frame loop down, and that tolerance is exactly what would
// let a renamed panel go dark in silence. This is the check that makes the tolerance safe.
const htmlSource = readFileSync(join(here, "index.html"), "utf8");
const domIds = new Set([...htmlSource.matchAll(/\bid="([A-Za-z0-9_-]+)"/g)].map((m) => m[1]));
const written = new Set([
  ...appSource.matchAll(/\bput\("([A-Za-z0-9_-]+)"/g),
  ...appSource.matchAll(/\btag\("([A-Za-z0-9_-]+)"/g),
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

/// Build the O:2H scene exactly as `loadPreset` builds it, optionally WITHOUT the H3
/// surface, and report what the engine did.
///
/// `ohKnots` is a BUDGET, not a shortcut. The O-H solve is dominated by a fixed setup cost
/// — measured at 17.0 s for 4 knots against 14.7 s for 16 — so the knot count buys almost
/// nothing here and costs a minute of CI at the 160 the page uses. What this gate is
/// asserting about that curve is that the engine SERVES it in a browser host, and eight
/// knots establish that as well as a hundred and sixty do. The blind arm skips the solve
/// entirely because its subject is the fence counter, which does not depend on it.
async function o2hScene({ withTrimer, ohKnots = 8, solveOH = true }) {
  const e = await freshEngine();
  e.holon_set_dims(1);
  e.holon_set_boundary(0);
  e.holon_set_census_enabled(1);
  e.holon_bank_clear();
  e.holon_bank_register(1);
  e.holon_bank_register(8);
  e.holon_table_generate(0.6, 12.0, 192);
  const oh = solveOH ? e.holon_bank_generate_pair(8, 1, ohKnots) : null;
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
want(served.oh === 1, "the O-H curve is served in the browser",
  `holon_bank_generate_pair(8,1) -> ${served.oh}`);

// 21 == PROVENANCE_REFUSED (16) + Refusal::SplitViolated (5). The page names this code in
// the text it shows the user, so the gate pins it: if the engine renumbers its refusals,
// the page's fence would go on explaining the wrong one and nothing else would notice.
want(served.oo === 21,
  `the O-O curve is REFUSED by the engine's in-browser split (code ${served.oo}, `
  + `${served.e.holon_bank_pair_n_det(8, 8).toExponential(2)} determinants past a limit of `
  + `${served.e.holon_bank_in_browser_det_limit()})`,
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

want(served.e.holon_water_loaded() === 0 && served.e.holon_trimer_surfaces() === 0,
  "the (O,H,H) surface is genuinely absent — the page's fence is a fact, not a choice");

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

// The repository root, FOUND rather than assumed. `join(here, "..", "..")` is right only
// while this file sits exactly two levels down, and it silently resolves to the wrong
// directory the moment the gate runs from a copy — which made every mutation test of this
// block fail with "artifact does not exist" instead of the defect it was probing. A check
// whose failures all look the same cannot tell you which one fired.
let repoRoot = here;
for (let up = 0; up < 6; up++) {
  try { readFileSync(join(repoRoot, ".git", "HEAD")); break; } catch { /* keep climbing */ }
  try { readFileSync(join(repoRoot, ".git")); break; } catch { /* worktrees: .git is a file */ }
  repoRoot = join(repoRoot, "..");
}

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
  }));
  want(bands.length === 4, `the ladder carries four bands (found ${bands.length})`,
    `parsed: ${bands.map((b) => b.band).join(", ")}`);

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

    if (state === "fenced") {
      const hasOwner = /owner:\s*"[^"]+"/.test(whole);
      const hasExit = /exit:\s*"[^"]{40,}"/.test(whole);
      want(hasOwner && hasExit,
        `fenced band "${band}" carries an owner and an exit`,
        `owner ${hasOwner ? "present" : "MISSING"}, substantive exit ${hasExit ? "present" : "MISSING"} `
        + "— the fence law requires both, and an exit too short to say anything is not one");
    }
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
    let certResolves = false;
    if (certificate) {
      const [relPath, lineNo] = certificate.split(":");
      try {
        const text = readFileSync(join(repoRoot, relPath), "utf8");
        const line = text.split("\n")[Number(lineNo) - 1] ?? "";
        // A certificate is a VERDICT, so the cited line must actually carry one. "CERTIFIED"
        // is the census's own word; a citation to a line that merely mentions the band would
        // pass a weaker check while establishing nothing.
        certResolves = /CERTIFIED/.test(line);
        if (!certResolves) {
          no(`band "${band}" cites a certificate at ${certificate} that carries no verdict`,
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

  const live = bands.filter((b) => b.state === "live").length;
  want(live === 1,
    `exactly one band is LIVE (${live}) — the others must fence rather than degrade`,
    "more than one live band means a coarse chart is being served that this engine does "
    + "not have, which is the tier-faking the ladder exists to forbid");
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

press.holon_set_boundary(2);
want(press.holon_pressure_defined() === 1,
  "on a periodic box the readout IS a pressure, and boundary mode 2 now reaches it");
want(Number.isFinite(press.holon_pressure()), "and it reads");

// Exports whose ABSENCE is the stated reason a panel is fenced. If one appears, the fence
// text on the page has become false and this gate says so. The failure message is an
// instruction, not a complaint.
const FENCE_JUSTIFYING_ABSENCES = {
  holon_set_pressure: "no setpoint door ships: WB-2.2's control IS the box (holon_box_scale); pressure is the readout, not a target",
  holon_phase_call: "the blind classifier (WB-5.5) is fenced on the page because none exists",
  holon_q_tet: "the order parameters (WB-5.5) are fenced on the page because none are computed",
  holon_water_table_begin: "the (O,H,H) surface is fenced on the page for want of an ABI door",
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
