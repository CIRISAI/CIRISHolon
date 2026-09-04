// The Water Workbench — the REAL engine.
//
// FSD-W1 (conformance/water_observatory/WORKBENCH_FSD.md) is the specification. This file
// replaces the WB-7.1 MOCK physics core (commits 84759ca / 2d0fc5e) with the Rust engine
// compiled to wasm32-unknown-unknown: `holon-render`, driven through its raw extern "C"
// ABI exactly as the atom viewer drives it. Nothing here integrates anything, invents a
// number, or interpolates a curve.
//
// WHAT CHANGED FROM THE MOCK, AND WHY EACH CHANGE IS LOAD-BEARING
//
//   `Math.random()` initial states       -> `holon_reset(n)`, a deterministic opener
//                                           (Fibonacci sphere + a derived unbinding
//                                           expansion). WB-5.4 replay is a property of
//                                           the engine, not a promise of this file.
//   450.0 harmonic + 0.015 LJ            -> `holon_table_generate`: STO-3G full CI on the
//                                           H2 curve, computed AT LOAD in the browser,
//                                           agreeing with a pinned 50-digit referee to
//                                           5e-15 hartree over 492 separations. WB-5.1.
//   hardcoded 104.5 deg / 0.096 nm       -> nothing. Molecules are DISCOVERED by the
//                                           engine's census; this file draws what the
//                                           bond criterion reports and defines no geometry.
//   `sin(performance.now())` as dE       -> `holon_drift()` against `holon_drift_bound()`,
//                                           with `holon_energy_gate()` as the verdict.
//   P^-0.05 box scaling as NPT           -> `holon_box_scale`, an affine rescale of
//                                           container AND contents whose cost is posted to
//                                           both ledger columns, with pressure as the
//                                           virial READOUT and its defined-flag honoured.
//                                           (This line denied the barostat's existence
//                                           until FENCES.md F-2 caught it: the barostat
//                                           landed and the sentence did not move. What is
//                                           fenced is only the SETPOINT door, and by
//                                           design — the control is the box. The denial is
//                                           paraphrased rather than quoted here because the
//                                           gate below matches the phrase itself, and for
//                                           the same reason the water panel does not quote
//                                           the causal claim it declines to make: restating
//                                           a false sentence to disown it still puts the
//                                           sentence on the page.)
//   "refinement patch" banner            -> a FENCE. There is no refinement patch either.
//   manifest claiming T6 certification   -> the manifest reports the SHA-256 of the wasm
//                                           bytes this page actually instantiated.
//
// THE TAG DISCIPLINE (WB-7.1). Every displayed quantity carries one of two tags and there
// is no third:
//
//   LIVE    the digits are a value this page read out of the engine THIS FRAME, and the
//           readout function is named in the panel's own `trace` field.
//   FENCED  the engine cannot serve this quantity. The panel renders the fence and the
//           REASON, and shows no digits at all. WB-5.2: never faked, never interpolated
//           across, never silently zeroed.
//
// SYNTHETIC does not appear in this file. A number either traces or it is fenced; the
// third option is the incident WB-7.1 was written from.

"use strict";

// ---------------------------------------------------------------- units
//
// CODATA, and the same values the engine's own headers carry. Conversions live here
// because the engine works in atomic units throughout and refuses to carry a display
// concern; every number crossing into a label goes through exactly one of these.

const BOHR_TO_M = 0.529177210903e-10;
const HARTREE_TO_J = 4.3597447222071e-18;
/// One atomic unit of time in femtoseconds (hbar / E_h). Pinned to `clock.rs`'s AU_TO_FS;
/// `boot()` checks the two agree by reading `holon_period_fs() / holon_period()` out of
/// the artifact rather than trusting this constant.
const AU_TO_FS = 0.024188843265857;
/// Boltzmann's constant in hartree per kelvin, and hydrogen's mass in electron masses.
/// Both are the engine's own values (`sim.rs::K_B`, `sim.rs::M_H`), repeated here only
/// because the ABI exports no reader for either; they are used for the gravity-vs-kT
/// exhibit and for nothing the physics depends on.
const K_B_HA = 3.166811563e-6;
const M_H_ME = 1837.152;

// ---------------------------------------------------------------- state

const State = {
  /// The wasm exports. Null until `boot()` succeeds; every readout guards on it.
  w: null,
  /// SHA-256 of the exact bytes instantiated, and their count. The manifest shows these
  /// rather than a commit id: a commit id is a claim about the build, the digest is the
  /// build.
  artifact: { sha256: "…", bytes: 0 },
  /// The declared device class (M-DEVICE-CLASS). WB-5.4 makes determinism a per-class
  /// property, so the class has to be stated before any replay claim is made.
  deviceClass: "wasm32-unknown-unknown/f64",

  booted: false,
  bootError: null,
  paused: false,

  /// The active scene preset. See `PRESETS`.
  mixture: "pure-h",
  /// What the preset's boot actually achieved, filled in by `loadPreset`.
  served: null,

  /// Requested atoms. Clamped by the engine to its own capacity; `atomsActual` is what
  /// came back and is what the panels report.
  atomsRequested: 12,
  atomsActual: 0,

  thermostatOn: true,
  targetK: 293.15,
  tempUnit: "K",

  /// The gravitational field, in MULTIPLES OF ONE G (WB-2.4). The engine takes atomic
  /// units; this is the number the slider carries, and `applyControls` multiplies by the
  /// engine's own `holon_g_earth()` rather than by a constant of this file's.
  ///
  /// The default is 1 G and not 0: WB-2.4 asks for 1 G downward at every scale, and a
  /// field this small changes nothing measurable at the atomic tier — which IS the
  /// exhibit. Measured: 4.05e-15 of kT for a hydrogen atom raised 1 nm.
  gravityG: 1.0,

  /// WB-2.4c — THE TILTED BUCKET. `tiltDeg` rotates the BOX relative to the world; the
  /// field stays world-down. The engine's box is axis-aligned, so "the box tilted by θ"
  /// is expressed as the field pointing at −θ from −y IN BOX COORDINATES, and the render
  /// rotates the box by −θ so the world looks level. That is not a trick standing in for
  /// the physics — it IS the physics, because a uniform field has no other content than
  /// its direction relative to the container.
  tiltDeg: 0,

  /// The barostat is the BOX (WB-2.2). This is the cumulative factor the user has applied,
  /// kept only so the panel can show how far from the reference box the scene is; the
  /// engine owns the box and this never drives it.
  boxScale: 1.0,
  lastScaleRefusal: null,

  /// 0 Walls · 1 Open · 2 Periodic. Exposed because it is the knob that makes two fences
  /// LIVE rather than theoretical: the virial is only a pressure on a wrapping box, and a
  /// wrapping box is the one that refuses gravity. A user can now watch each fence fire
  /// for its own reason instead of reading that it would.
  boundary: 0,
  gravityRefused: false,

  /// The governor's user bias (WB-2.3). Multiplies the engine's DERIVED base sim-speed;
  /// it never touches dt, so accuracy is not on this slider.
  govBias: 1.0,
  baseSimSpeed: 0,

  /// Measured, never assumed (WB-1.4). `rate` is delivered simulated femtoseconds per
  /// wall second, computed from `holon_time()` deltas over a wall-clock window.
  rate: { fsPerSec: 0, pctRealtime: 0, fps: 0 },
  clockWindow: { t0: null, simFs0: 0, frames: 0 },

  /// The camera, and its TARGET owns the view centre (lead's ruling, derived from the
  /// FSD rather than chosen). The centre is AIM, and aim belongs to the observer: the
  /// acuity law is written from the observer's side, and the seeding rule pins one holon
  /// "near the center" — near where you are LOOKING. The hand is physics and aiming has
  /// no physical meaning, so letting the hand place the centre would couple the two axes
  /// the two-box law separates. The ratio correction coupled SIZE through the quotient,
  /// deliberately and only that.
  camera: { yaw: 0.35, pitch: 0.25, distance: 2.8, fov: 45.0, target: null },

  /// THE ZOOM, and it is a RATIO rather than a length (FSD-W2, the two-box law). The
  /// scene box is the WORLD box divided by this number, so 3× zoom on a 1 km world and
  /// 3× zoom on a 0.5 km world are different scene-box sizes — the view is coupled to
  /// the world through the ratio and only through the ratio. The hand stretches the
  /// world and the scene box follows; the zoom changes the quotient and the physics does
  /// not notice.
  zoom: 1.0,

  /// The scene-event log (append-only). Every holon crossing a scene-box face is recorded
  /// here as it happens. Nothing is destroyed by a crossing — the WORLD box keeps
  /// simulating every holon it has, which is exactly what makes its pressure whole-only
  /// and zoom-invariant — so this is a record of what the VIEW is showing, not of what
  /// the physics is doing. A scene that quietly drew a fraction would be a picture that
  /// misrepresents its own physics.
  sceneLog: [],
  sceneMembership: null,

  /// The hand (WB-4). `grabbed` is the engine's own index, so the receipt below and the
  /// atom on screen cannot disagree about who is being pulled.
  hand: { grabbed: -1, screenX: 0, screenY: 0, radiusBohr: 0 },

  /// The determinism exhibit (WB-5.4). Two runs of the same seeded scene under the same
  /// device class must produce the same digest.
  replay: { last: null, prev: null, matched: null },

  /// The native referee's pinned reference solve (WB-10.2), read from `law_probe.json`
  /// beside this page. Null until it loads and null forever if the engine lane has not
  /// written it yet — in which case the bit-identity row PENDS rather than passing.
  lawProbe: null,
  lawProbeWhy: null,

  /// The last atom-band solve and WHEN it was taken. The engine keeps one result at a time
  /// and the page runs it on a throttle, so the readouts card shows the frame it belongs to
  /// rather than letting a solve from 400 frames ago read as the current one.
  atomBand: { atom: -1, atMs: -Infinity, atFrame: 0, atTimeFs: 0, exit: 4 },
};

// ---------------------------------------------------------------- scene presets (WB-3.1)
//
// Composition is IDENTITY, not a slider: every entry here is a scene RESET. What each
// preset can honestly serve is not declared here — it is MEASURED at load by
// `loadPreset`, which asks the engine for each curve and records what it said. A preset
// whose curves the engine refuses does not fall back to anything; it fences.

// THE TWO DOORS TO A CURVE, AND WHY THIS FILE PICKS ONE.
//
// The engine offers two routes to a pair potential and they are NOT interchangeable in
// price, which is a fact about this build that had to be measured rather than read:
//
//   holon_table_generate       H2 only, through holon_chem::stream_table's bespoke s-only
//                              path.  0.16 ms/knot, linear, measured in Chromium on the
//                              development machine over 64/192/384 knots.
//   holon_bank_generate_pair   the general N-centre route.  For the SAME H-H curve:
//                              ~0.5 s fixed + ~58 ms/knot, i.e. 7.0 s at 160 knots —
//                              about 90x the first door for physics that agrees with it
//                              to six digits.  For O-H: ~15 s FIXED, 55 s at 160 knots.
//
// So H-H always comes through the cheap door, and it is not a shortcut: the two agree on
// R_e and D_e to the digits the page displays, and the expensive door's own header says
// the difference between them is provenance bookkeeping, not physics. Anything the cheap
// door cannot serve is PRICED — offered with the engine's own declared determinant count
// and paid only when the user asks, never spent silently on a main thread that would
// freeze for a quarter of a minute with no explanation. M-CHEAPER-THAN-ITS-PRICE is a
// runtime law (WB-5.2), and a page that blocks for 55 s has not obeyed it just because
// the number it eventually shows is right.

const PRESETS = {
  "pure-h": {
    label: "Pure H",
    sub: "H₂ gas — fully banked",
    species: [1],
    /// Every unordered pair the scene can meet, in the order they are asked for. H-H is
    /// absent because it arrives through the cheap door before this list is walked.
    pairs: [],
    /// The homonuclear H3 surface generates IN THE BROWSER: nine determinants a node over
    /// 14,157 nodes. Measured at 4.8 s here, which is why the boot paints a stage line.
    trimer: "generate",
  },
  "o-2h": {
    label: "O : 2H",
    sub: "the water-formation experiment",
    species: [1, 8],
    pairs: [[8, 1], [8, 8]],
    /// The H3 surface IS generated here, and that is not decoration: an O:2H box contains
    /// real H-H-H triples and they are served. It is also what makes the fence real —
    /// the engine's three-body pass returns early when no surface at all is loaded, so
    /// without H3 the O-bearing triples would be skipped silently and
    /// `holon_fence_untabulated` would read a zero that meant "never looked".
    trimer: "generate",
    /// Triples the engine will meet and refuse are DERIVED at load (`untabulatedTriples`):
    /// the (O,H,H) surface is 441 determinants a node — about a thousand times an H3 node
    /// — so it is computed on the mesh, SHIPPED as a text artifact and pushed through the
    /// water door (`holon_water_table_alloc` / `holon_water_table_load`); (O,O,H) and
    /// (O,O,O) are not tabulated anywhere yet. Each encounter is refused and COUNTED.
  },
  // ONE OXYGEN, AND THE REASON IT EXISTS.
  //
  // The O:2H preset above steps exactly when both of its O-bearing curves are in the bank:
  // (O,H) and (O,O) are past the browser's split (the engine's own cost model, re-measured
  // on the shipped wasm: the (O,O) curve is minutes, not a page load), so both arrive as
  // committed artifacts through the pair door, digest-pinned in `SHIPPED`. Until an
  // artifact is in the tree the preset is a fence that names it, never a frozen box
  // reported as settling.
  //
  // A box with EXACTLY ONE oxygen never forms an (O,O) pair, so it needs only (O,H) and
  // runs — real O-H chemistry, with (O,H,H) served from the shipped table and the O-bearing
  // fences it cannot form never firing. It is labelled for what it is: not the water
  // stoichiometry, and not offered as a substitute for it.
  "one-o": {
    label: "1 O + H",
    sub: "one oxygen — the O-bearing scene with every triple it can form tabulated",
    species: [8, 1],
    composition: "single-o",
    pairs: [[8, 1]],
    trimer: "generate",
  },
  "pure-o": {
    label: "Pure O",
    sub: "would ride the (O,O,O) surface",
    species: [8],
    pairs: [[8, 8]],
    trimer: "generate",
  },
};

/// The O-bearing triples a preset's scene can form that this build has no surface for.
///
/// DERIVED from the composition and from what the water door actually read, never
/// declared per preset: a declared fence list is how (O,H,H) stayed "fenced" on the page
/// for a build whose parser could have read the table. (O,O,H) and (O,O,O) are not
/// tabulated anywhere yet, so they are fenced wherever two oxygens can meet.
function untabulatedTriples(preset, waterServed) {
  const hasO = preset.species.includes(8);
  const hasH = preset.species.includes(1);
  const manyO = hasO && preset.composition !== "single-o";
  const out = [];
  if (hasO && hasH && !waterServed) out.push("(O,H,H)");
  if (manyO && hasH) out.push("(O,O,H)");
  if (manyO) out.push("(O,O,O)");
  return out;
}

/// Species drawing data, loaded from `species_palette.json`. `radius_bohr` there is
/// DERIVED by the engine (half the element's own computed homonuclear separation), so the
/// picture's proportions trace to the same solver as the physics.
let PALETTE = new Map();

// ---------------------------------------------------------------- dom

const $ = (id) => document.getElementById(id);
const UI = {};
function bindUI() {
  for (const el of document.querySelectorAll("[id]")) UI[el.id] = el;
}

/// Write text into an element only if it exists, so a panel removed from the HTML does
/// not take the frame loop down with it.
function put(id, text) {
  const el = UI[id];
  if (el && el.textContent !== text) el.textContent = text;
}

/// Set a panel's honesty tag. `trace` names the export the digits came from, and lands in
/// the element's tooltip — WB-7.1 asks that a live number be traceable, and a name in the
/// title attribute is the cheapest form of that which survives into the deployed page.
function tag(id, kind, trace) {
  const el = UI[id];
  if (!el) return;
  el.dataset.tag = kind;
  el.textContent = kind === "live" ? "LIVE" : "FENCED";
  el.title = kind === "live" ? `traces to ${trace}` : trace;
}

/// One row of the ladder's readouts card, with its OWN tag beside its own digits.
///
/// The panel-level `tag()` above is not enough here and the reason is the point of the
/// card: three rows of the nucleus band are declared measured inputs, three are waiting on
/// an export that is still being built, and two are page arithmetic. One tag over the lot
/// would have to be the weakest of them, and a reader would have no way to tell which digit
/// was which. So the tag is per row, it is rendered next to the number rather than hidden in
/// a tooltip (`styles.css` draws it from `data-tag`), and a PENDING row is passed no digits
/// at all — `text` is the name of the export it is waiting for.
///
///   live      read out of the engine this frame
///   declared  a measured input, from the committed artifact named in `trace` (WB-1.7)
///   computed  page arithmetic over inputs that are themselves live or declared
///   pending   the export that serves this row is not in the artifact yet; no digits
function descField(id, kind, text, trace) {
  const el = UI[id];
  if (!el) return;
  el.dataset.tag = kind;
  el.textContent = text;
  el.title = trace || "";
}

// ---------------------------------------------------------------- the record (WB-9.6)
//
// A THIRD TAG, and it earns its place rather than diluting the discipline. LIVE means the
// digits came out of the engine this frame; FENCED means the engine cannot serve them.
// These numbers are neither: they were measured by an instrument that is not this page,
// on runs that are not this scene, and they are CITED. Calling them LIVE would be a lie
// about where they came from; fencing them would hide a result that exists. So RECORD, and
// every RECORD figure carries the artifact and line it came from — which `smoke.mjs`
// then VERIFIES by reading that file, so a citation cannot outlive the number it cites.
//
// Only COMMITTED artifacts may be cited. The `--de4=off` control has run and its census
// is banked but not yet in the repository, so its figure is deliberately absent below: a
// published page must not assert a number that a clean checkout cannot check. When it
// lands the row appears and the gate starts checking it.

const RECORD = {
  window: {
    value: "834 fs",
    what: "the pre-staked holding window",
    cite: "conformance/water_observatory/census_mixed_fenced.log:3",
    note: "frozen in CENSUS_PREREG.md before the instrument that measures it existed.",
  },
  waterA: {
    value: "893.8 fs",
    what: "OH₂ held, CERTIFIED-STRICT — fenced arm",
    cite: "conformance/water_observatory/census_mixed_fenced.log:233",
    note: "seed 0x…25, block 0x0a08, 1073 frames — 72.3% of the run, rms 0.779 bohr, "
      + "separation variance 0.199, NAMED. Control rate 0.000 against a pool of 111 "
      + "same-composition candidates — the denominator matters, because a bare 0.000 "
      + "reads like a default. Clears the window by 7%.",
  },
  waterB: {
    value: "923.9 fs",
    what: "OH₂ held, CERTIFIED-STRICT — the four-body term switched OFF",
    cite: "conformance/water_observatory/census_de4_off.log:22",
    note: "seed 0x…22, block 0x0062, 1109 frames — 85.8% of the run, rms 1.103 bohr, "
      + "NAMED, control 0.000 against the same 111-candidate pool. Clears the window "
      + "by 11%.",
  },
  absent: {
    value: "0",
    what: "dE₄ evaluations in that run — the term is measured absent, not just unflagged",
    cite: "conformance/water_observatory/PROVENANCE_de4_arms.md:43",
    // A DISPLAY value of "0" is honest and almost worthless to a checker: "0" occurs on
    // most lines of most files, so citing it would pass without establishing anything.
    // `match` is what the gate looks for instead — the arm row's own momentum figure,
    // which is distinctive to exactly this line.
    match: "3.84e-5",
    note: "`Sim::de4_eval_count` is incremented by the physics itself, so a reading of "
      + "exactly zero is functional proof the four-body term never fired. A symbol-table "
      + "check could not establish that, because the symbol is inlined away.",
  },
  sameCommit: {
    value: "21e6be3",
    what: "the control's commit, with its binary hash",
    cite: "conformance/water_observatory/PROVENANCE_de4_arms.md:13",
    note: "binary sha256 462045fe…, own target directory, detached worktree. This sidecar "
      + "exists because the run log carried no provenance line, and 'same commit' is the "
      + "entire content of a one-variable control.",
  },
  ruling: {
    value: "FALSE on this seed",
    what: "the causal claim, RULED — and by the control alone",
    cite: "conformance/water_observatory/CENSUS_RESULTS.md:830",
    note: "§12.3's branch (b), staked before the data existed: BOTH arms certify block "
      + "0x0062 — the same three atoms — CERTIFIED-STRICT, control 0.000/111 in each. The "
      + "ruling rests on the CONTROL and needs nothing from the treatment arm: arm B has "
      + "dE₄ evaluations 0, conserves momentum at 3.84e-5 of bound, and certifies water "
      + "strict at 923.9 fs.",
  },
  notRuled: {
    value: "2599.8 fs vs 923.9 fs",
    what: "what the four-body term DOES — explicitly NOT ruled",
    cite: "conformance/water_observatory/CENSUS_RESULTS.md:819",
    // The DISPLAY value pairs both numbers because the comparison is the point; the
    // artifact's row carries them in its own columns, so the checkable token is the one
    // that is distinctive to that line.
    match: "2599.8",
    note: "the treated arm's molecule held longer, and that difference may NOT be "
      + "attributed to the term: the treated run ALONE leaves its declared 2D plane, "
      + "reaching 11.49 bohr against a 12.0 half-depth from frame 4230 — just after the "
      + "term first fires — while seventeen other trajectories hold z bit-exactly for "
      + "20,000 frames. The one-variable design was defeated by the treatment producing a "
      + "second variable, so a bent triatomic had room the control could not reach.",
  },
  legB: {
    value: "NOT CLOSED",
    what: "Leg B, the fiber-invariance test — on BOTH arms",
    cite: "conformance/water_observatory/census_de4_off.log:37",
    note: "124 witness pairs here, 158 on the fenced arm. Each molecule is certified as a "
      + "held THING; the full partition view it sits in is not a closed one. Quoting only "
      + "the first would be quoting half a verdict.",
  },
};


// ---------------------------------------------------------------- the fence register
//
// R6 of the retirement battery, at the lead's ruling: FENCES.md is the SINGLE register,
// every fence the page DISPLAYS gets a row there, and the page carries owner + exit and
// CITES the row. The citation is by ROW ID rather than by line, because an id survives the
// register being reordered and a line number does not — the same reason the misfit gate
// greps for `M-` ids instead of offsets.
//
// Most of these were already registered by the bank-fences lane before I came to cite
// them, which is the register working: P13 the classifier, P14 the order parameters, P16
// the refinement patch, C3 the in-browser split, P2 the untabulated-triple counter. Where
// a page-local decision has no row, it says so and names what would earn one — an
// UNREGISTERED marker that the gate treats as a failure once the row exists is the same
// two-directional shape as the band certificates.
const FENCE_REGISTER = {
  splitViolated: {
    owner: "holon-chem / atomworld",
    exit: "the mesh solve — this is a page-load budget, not a capability limit",
    register: "C3",
  },
  unpaidCurve: {
    owner: "workbench-engine",
    exit: "press SOLVE, or ship the curve as a committed artifact through the pair door",
    register: "C3",
  },
  artifactRefused: {
    owner: "workbench-engine",
    exit: "re-emit the artifact with the engine that ships (emit_pair_tables / s2_table), "
      + "or re-pin SHIPPED to the committed bytes; a digest that does not match is never "
      + "served",
    register: "C3",
  },
  untabulatedTriples: {
    owner: "ozone / atomworld lane",
    exit: "the (O,O,O) tabulation lands and its certification upgrades the fence to served",
    register: "P2",
  },
  trimerRefused: {
    owner: "workbench-engine",
    exit: "the H₃ generator declining its own grid is an error path, not a limit; if it "
      + "fires the grid request is wrong",
    register: "P2",
  },
  cannotStep: {
    owner: "workbench-engine",
    exit: "serve every pair the scene's atoms can meet — this fence is DERIVED from the "
      + "pair fences above it and lifts when they do",
    register: "C3",
  },
};

/// The panels this engine cannot serve at all. Data rather than markup since R3 went full:
/// the gate reads this list and requires the same owner/exit/register of every entry that
/// the runtime fences carry. As prose in the HTML it was uncheckable, and "a citation pass
/// over every fence the page displays" cannot mean "over the ones that were convenient".
const NOT_SERVED = [
  {
    what: "Blind phase classifier (WB-5.5)",
    why: "the phase call is OWED. The census computes phase fractions natively today; what "
      + "is missing is the ABI door that brings them to the page, which is a door and not "
      + "a discovery. Until it lands the panel shows nothing rather than a guess — the mock "
      + "printed \"LIQUID WATER · 99.8%\" and that number had no source.",
    owner: "workbench-engine", register: "P13",
    exit: "a classifier that reads the scene rather than the preset that launched it",
  },
  {
    what: "Order parameters q_tet, Q₆, ⟨H-bonds⟩, MSD",
    why: "the lens stack that computes these runs natively in the census — q_tet, Q₆, "
      + "H-bond counts and MSD, with their own refusals where a lens does not apply. The "
      + "work owed is the door, not the lenses.",
    owner: "workbench-engine", register: "P14",
    exit: "the lens stack the census already runs natively, exposed through the ABI",
  },
  {
    what: "Local refinement patch (WB-1.2 / WB-4.4)",
    why: "refinement is owed to the mesher, and the closure-budget signal it would trigger "
      + "on is already measured per row. The mock showed a banner with no solve behind it; "
      + "what is missing is the solve, and it has an owner.",
    owner: "mesher", register: "P16",
    exit: "a refinement patch that opens on a measured closure-budget breach",
  },
];

// ------------------------------------- the scale ladder (FSD-W3 §11.2, superseding §9c)
//
// The site's hero is the zoom axis itself, and FSD-W3 §11.2 sets its ends: 1 km × 1 km at
// the top, the NUCLEUS at the bottom. Every band is PRESENT and is exactly one of two
// things — LIVE on its certified chart, or FENCED with its debt, its owner and its exit in
// the present tense. Nothing in between, and no band ever fakes a tier.
//
// A FENCE IS A BUG UNDER REPAIR (operator's law). The rungs on this ladder are a work
// queue, not a display of scruple: each fenced band names the build that is paying it off,
// and the band flips when that build lands its warrant — never when the sentence improves.
//
// THREE KINDS OF WARRANT, and they are not interchangeable:
//
//   the coarse bands (cube, fluid element, H-bond network) flip on a NODE-G CLOSURE
//   CERTIFICATE — a coarse view of the dynamics beneath them, certified by the census.
//   That is §9c's band-flip law and it is gated in both directions.
//
//   the fine bands (atom, nucleus) are not coarse views of anything, so no closure
//   certificate applies to them. They flip on their EXPORTS: the quantities they display
//   are served by the engine or they are not, and `liveWhen` names exactly which. Until
//   the rebuilt wasm carries them the band renders FENCED — PENDING with the export named,
//   and shows no digits. `bandLiveness` below is that rule, and it is what the page runs.
//
//   the fold below the atom is fenced on physics nobody has yet: node GF2's hadron box.
//
// §11.2's bottom row — the gauge vacuum below the nucleus (W2) — is deliberately NOT on
// this ladder. The operator sequenced it after W, and a band on the page for work that has
// not been scheduled is a promise wearing a fence's clothes.
//
// THE ZOOM IS DE-ALLOCATION, NOT A HANDOFF. There is no transition machinery and none is
// owed: a holon that is not load-bearing for the scene releases its members' fine degrees
// of freedom, and "load-bearing" is not a heuristic but the MEASURED per-row closure
// defect — a row reading ~0 is autonomous BY MEASUREMENT and its composite carries it
// exactly on grain boundaries, while a row being buffeted, grabbed or coupled into the
// visible region scores badly and keeps its fine allocation. Re-allocation on zoom-out is
// the same accounting-only event in reverse.
//
// WHAT THIS PAGE DOES AND DOES NOT DO, stated because the difference is the whole honesty
// of the panel: it REPORTS the criterion — the live per-row defects that decide
// allocation — and it performs no de-allocation, because none is implemented. A panel
// that displayed a budget being reclaimed while nothing was reclaimed would be the
// synthetic-telemetry shape at a new altitude.

const LADDER = [
  {
    band: "the cube",
    scale: "1 km",
    lengthM: 1.0e3,
    runs: "the continuum face of the ladder — its DYNAMICS fenced, its hydrostatic column "
      + "a live readout",
    state: "fenced",
    owner: "GANTT node G",
    exit: "this face goes live as the rungs beneath it certify — rung 1's carrier and "
      + "rung 2's carrier-v2 are both in build, and this face is their composition. A "
      + "kilometre of water is ~3×10³¹ molecules; the acuity law is why that is a scale "
      + "to stand in front of rather than a number to simulate.",
    cite: "conformance/water_observatory/WORKBENCH_FSD.md:379",
    // ONE THING THIS BAND SERVES WHILE ITS DYNAMICS ARE FENCED, and §11.2 grants it by
    // name: the hydrostatic column, ρ g h. It is not a dynamics readout and is never
    // offered as one — it is arithmetic on constants this page has already measured (the
    // scene's own density, the engine's own g), which is exactly the gravity exhibit
    // WB-2.4a makes: what changes across the tiers is not the field but whether the
    // quantity that matters is a per-particle energy or a sum over the column. The digits
    // live in one place — the readouts card below — so this row points at them rather
    // than restating them, and the band stays FENCED.
    readout: "ρ g h, the hydrostatic column, in the ladder readouts card below — "
      + "arithmetic on measured constants, never a dynamics readout",
    readoutCite: "conformance/water_observatory/WORKBENCH_FSD.md:584",
  },
  {
    band: "fluid element",
    scale: "~µm+",
    lengthM: 1.0e-6,
    runs: "the carrier that certifies this band is BEING BUILT",
    state: "fenced",
    owner: "GANTT node G, rung 2 — banked NOT CERTIFIED, branch (d) of its own freeze",
    // A MEASURED FENCE, which is a better fence than the one it replaces: "no certified
    // chart exists" is a state, and a quantified boundary is a fact. Rung 2 spent the
    // compute and came back with numbers rather than a shrug.
    //
    // The scissor: coarse cells hold atoms but nothing crosses their faces, and the only
    // grid that transports averages 0.5 atoms per cell. A 1 µm patch at the certified
    // density is 5.95e6 atoms against a 12-atom certified scene — 4.96e5× — and a
    // 16-atom trajectory-format cap besides.
    //
    // THE EXIT IS UNDETERMINED, AND THAT IS THE PRE-COMMITTED ANSWER. Two occupancy points
    // with overlapping ranges, five orders from the band; the freeze's own rule forbids
    // extrapolating from them and pre-committed that UNDETERMINED beats a fitted trend.
    // Naming it as undetermined is what keeps this a fence rather than architecture: the
    // successor ROUTES are named even though the distance is not.
    exit: "carrier-v2 is in build: trajectory format v2 past the 16-atom cap, genuine-3D "
      + "≥400-atom scenes on the threaded MD path, dims MEASURED rather than declared. "
      + "Rung 2's numbers are the requirement it must beat — a 1 µm patch is 5.95e6 atoms "
      + "against a 12-atom certified scene, and the occupancy and transport conditions "
      + "scissor: coarse cells hold atoms but nothing crosses their faces, and the only "
      + "transporting grid averages 0.5 atoms per cell. The DISTANCE is undetermined and "
      + "pre-committed to be reported that way — two occupancy points five orders from the "
      + "band, which the freeze forbids extrapolating — so the build is named and the "
      + "estimate is not.",
    cite: "conformance/water_observatory/WORKBENCH_FSD.md:378",
    // The measurement that says the fence is real rather than a gap in the schedule.
    measuredBy: "conformance/water_observatory/RUNG2_RESULTS.md:214",
    // AND THE POSITIVE HALF, because a fence is not the whole finding. Hydrodynamics'
    // premise — that momentum is spatially coherent over a cell — was MEASURED REAL at the
    // 5.8 bohr scale (+0.598 median, 7/7 clearing the bar). What is out of reach is the
    // certificate, not the physics. A page that showed only the refusal would be reporting
    // half of what the lane found.
    positive: "the momentum field IS spatially coherent at 5.8 bohr — +0.598 over the "
      + "scrambled control, 7/7 seeds. Hydrodynamics' premise measures real here; it is "
      + "the CERTIFICATE that is out of reach, not the phenomenon.",
    positiveCite: "conformance/water_observatory/RUNG2_RESULTS.md:47",
  },
  {
    band: "H-bond network",
    scale: "~10 nm",
    lengthM: 1.0e-8,
    runs: "the carrier that certifies this band is BEING BUILT",
    state: "fenced",
    owner: "GANTT node G, rung 1 — banked branch (D), NOT certified",
    // A FENCE IS A BUG UNDER REPAIR, NEVER CONTENT (operator's law). What goes on screen
    // is the DEBT, its OWNER, and THE BUILD PAYING IT, in the present tense. The measured
    // numbers are still here — they are the requirement the build has to beat — but they
    // are the specification of the work, not a display of why refusing was clever.
    exit: "the physics ladder and the T3 scale-up are the named unblockers; rung 1's "
      + "readings are the bar that build must clear — 70 chart readings in which the two "
      + "conditions a certified tier needs are EXACTLY DISJOINT: 36 inside the closure "
      + "budget and all 36 VOID by anti-vacuity, 32 clearing anti-vacuity and none inside "
      + "the budget, zero doing both.",
    cite: "conformance/water_observatory/WORKBENCH_FSD.md:377",
    measuredBy: "conformance/water_observatory/RUNG1_RESULTS.md:19",
    // The mechanism, which is the display-worthy part: the boundary is ALIGNMENT, not
    // presence and not proximity. That is a result about water, not a note about us.
    positive: "the molecules are there and the proximity is there — two or more separate "
      + "oxygen-bearing molecules in 84–99.8% of frames on eight of ten trajectories, "
      + "sitting within hydrogen-bonding distance for essentially the whole run. What is "
      + "missing is ALIGNMENT: frames carrying even one inter-molecular H-bond number "
      + "0–18 out of 20,000. The boundary is orientation, and that is a measurement.",
    positiveCite: "conformance/water_observatory/RUNG1_RESULTS.md:51",
  },
  {
    band: "molecular",
    scale: "~nm",
    lengthM: 3.0e-10,
    runs: "the live engine, full physics ladder",
    state: "live",
    cite: "conformance/water_observatory/WORKBENCH_FSD.md:376",
    // THE CERTIFICATE, not the specification. `cite` above points at the FSD line that
    // says this band SHOULD be live; that is a plan, and a plan is not a verdict. This
    // points at the census's own CERTIFIED-STRICT row — the molecular tier's closure test,
    // passed, on a banked trajectory. The gate requires it of every live band and refuses
    // to let a band be live without it, which is what makes "fenced -> live" a flip that
    // cannot be performed by editing one word.
    certificate: "conformance/water_observatory/census_mixed_fenced.log:233",
    // WHICH NODE'S CERTIFICATE, and it is load-bearing rather than provenance decoration.
    // A band goes live ONLY on a node-G closure certificate — a coarse view of the
    // dynamics BENEATH it, certified by the census. Node LG's lattice-gas tier is
    // certified on its OWN dynamics: real, banked, and NOT a band state, because running
    // physics that is not the certified coarse truth of THIS water is the fake §9c bans.
    // Without this field an LG bank would read as "the rung has landed and the flip is
    // owed" and the gate would demand a flip nobody is entitled to.
    certNode: "G",
    certifiedBy: "the closure census — OH₂ held 893.8 fs, CERTIFIED-STRICT, past the "
      + "pre-staked 834 fs window",
  },
  {
    // --- THE FINE BANDS. Everything below the molecular band is reached through a PICKED
    // atom (WB-1.6) and pinned ONE at a time by the acuity law. These bands hold no
    // closure certificate and are not entitled to one: a closure certificate certifies a
    // COARSE view of the dynamics beneath a band, and there is no coarse view here — the
    // atom band is the dynamics, solved. What it needs instead is the engine's own
    // arithmetic, through the exports `liveWhen` names.
    band: "atom",
    scale: "~Å",
    lengthM: 5.3e-11,
    runs: "ONE atom pinned by the acuity law — H or O, in a molecule or free — its "
      + "electronic structure solved by the lane engine IN THE PAGE (STO-3G FCI, the same "
      + "arithmetic as native, gated bit-identical against the native referee)",
    state: "export-gated",
    pinned: true,
    liveWhen: [
      "holon_atom_in_molecule", "holon_atom_band_solve", "holon_atom_band_energy",
      "holon_atom_band_n_electrons", "holon_atom_band_residual", "holon_atom_band_exit",
    ],
    owner: "lead (engine) — WB-10.1 / WB-10.2",
    // STATED AS A CONDITION, not as a build in progress — and it said the latter until the
    // doors landed, which made it stale within the hour. A fence names the build paying it
    // off; a band that is LIVE owes nothing and must not describe work as pending. The
    // gate now checks both directions of that, because only one of them was checked and
    // the wrong sentence was the one it let through.
    exit: "the atom band is live exactly when the shipped wasm carries `holon_atom_band_*` "
      + "and `holon_atom_in_molecule`, and fences by name — no digits — on any artifact "
      + "that does not. Its bit-identity row is green only while `law_probe.json` sits "
      + "beside this page with the digest `tests/wasm_law.rs` pinned natively.",
    cite: "conformance/water_observatory/WORKBENCH_FSD.md:588",
    buildCite: "conformance/water_observatory/WORKBENCH_FSD.md:619",
    ganttCite: "GANTT.md:108",
  },
  {
    band: "nucleus",
    scale: "~fm",
    lengthM: 2.7e-15,
    runs: "the nucleus of that atom as the deepest OBJECT this page carries: Z, isotope, "
      + "mass, nuclear spin and charge radius — DECLARED, MEASURED INPUTS the "
      + "Hamiltonian never computes (WB-1.7) — and its thermal de Broglie wavelength "
      + "at the scene's own measured temperature, COMPUTED in closed form by the engine",
    state: "export-gated",
    pinned: true,
    liveWhen: [
      "holon_nucleus_mass_u", "holon_nucleus_spin2", "holon_nucleus_charge_radius_fm",
      "holon_nucleus_thermal_wavelength_bohr",
    ],
    owner: "lead (engine) — WB-10.1",
    exit: "the nucleus is a DECLARED table, `holon_chem::elements::NUCLEI` (spin and "
      + "charge radius with their sources), served by the `holon_nucleus_*` doors under "
      + "`tests/nucleus.rs`; this band is live exactly when the shipped wasm carries those "
      + "doors, and fences by name — no digits — on any artifact that does not. Z and the "
      + "isotope name come from the committed species table and wear DECLARED, which is "
      + "what WB-1.7 asks of a measured input.",
    cite: "conformance/water_observatory/WORKBENCH_FSD.md:589",
    declaredCite: "conformance/water_observatory/WORKBENCH_FSD.md:598",
    buildCite: "conformance/water_observatory/WORKBENCH_FSD.md:618",
    ganttCite: "GANTT.md:108",
  },
  {
    // THE FLOOR, AND IT DRAWS NOW. This was a fence whose stated exit was node GF2's
    // three-dimensional hadron box. That exit CLOSED on 2026-09-04: the physics it was
    // reaching for is prior art (see `priorArt`), so the band went live on what this engine
    // can honestly compute instead — the EXACT colour-singlet ground state of one-flavour
    // QCD in 1+1 dimensions, solved in the page with no approximation, and the quark
    // density along the chain is the picture.
    band: "the fold below the atom",
    scale: "< 1 fm",
    lengthM: 8.4e-16,
    runs: "one baryon as a COLOUR-SINGLET bound state of three quarks, solved EXACTLY in "
      + "the page on the colour-lane determinant engine — the same arithmetic as native, "
      + "no approximation and no truncation. What is drawn is the quark density along the "
      + "chain; the difference between the B = 0 sea and the B = 1 state is one baryon's "
      + "worth of quarks, and where it sits is what this band shows. The baryon mass "
      + "(E1 - E0) / 2*sqrt(x) is read off the two solves when both are in the same box.",
    state: "export-gated",
    pinned: true,
    liveWhen: [
      "holon_hadron_solve", "holon_hadron_energy", "holon_hadron_occ",
      "holon_hadron_baryon_mass", "holon_hadron_n_det", "holon_hadron_exit",
      "holon_hadron_margin", "holon_hadron_dim_for", "holon_hadron_max_det",
    ],
    owner: "lead (engine) — the sub-atom band",
    // THE LIMIT, in the band and not only in the source. This is one space dimension.
    limit: "ONE SPACE DIMENSION. There are no transverse gluons here: the gauge field is "
      + "not dynamical, Gauss's law eliminates it, and the linear confining potential is "
      + "built in rather than emergent. No glueballs, no asymptotic freedom in the "
      + "four-dimensional sense. This is a MODEL that shares confinement and colour-singlet "
      + "structure with the real thing. It is not the proton.",
    // THE PRIOR ART, credited by name. Both did this physics before us, on cluster
    // hardware, and NEITHER PAPER REPORTS ITS RUNTIME — so no speed comparison is made
    // here or anywhere, in either direction.
    priorArt: "SU(3) lattice gauge theory in 1D with gauge-invariant matrix product states: "
      + "P. Silvi, Y. Sauer, F. Tschirsich and S. Montangero, Phys. Rev. D 100, 074512 "
      + "(2019) — the finite-density phase diagram, all phases colourless, and multi-baryon "
      + "bound states including the deuteron. And one-flavour SU(2) and SU(3) QCD2 with "
      + "DMRG: T. Hayata, Y. Hidaka and K. Nishimura, arXiv:2311.11643 — equation of "
      + "state, chiral condensate and quark distribution functions, SU(3) at 48 sites and "
      + "bond dimension 500 on RIKEN cluster machines. What is ours here is the engine and "
      + "the exactness in a browser, not the physics. NEITHER paper states its compute "
      + "time, so we do not know it and claim nothing about it.",
    // The COST, priced before it is spent: the sector is C(N, n_q/3)^3 and the door refuses
    // above its cap by name rather than hanging the page on a zoom gesture.
    // STATED AS A CONDITION, not as a build: the doors are in the shipped artifact, so this
    // band renders LIVE and owes nothing. It said "is being rebuilt" until the artifact
    // carried them, and the gate caught that within the minute — a fence that outlives its
    // debt is the page telling viewers about an absence that ended.
    exit: "the sub-atom band is live exactly when the shipped wasm carries `holon_hadron_*`, "
      + "and fences by name — no digits — on any artifact that does not. The door prices "
      + "every sector before solving it (`holon_hadron_dim_for`) and REFUSES above "
      + "`holon_hadron_max_det`: 3,375 determinants at N = 6 B = 1 is instant, 175,616 at "
      + "N = 8 runs, 9.3 million at N = 10 is refused with code 6.",
    cite: "GANTT.md:103",
    measuredBy: "TIERS.md:59",
    positive: "the first rung is MEASURED: SCHWINGER-4's residual interaction between two "
      + "screened static pairs decays at the banked meson mass to 0.6% — Fold II's "
      + "first measurement below the atom, 1+1D only.",
    positiveCite: "TIERS.md:59",
  },
];

/// THE ACUITY LAW (§9c): the observer's resolution bounds the allocation, and the seed is
/// ONE. A band's population is what the current view can actually distinguish — zero while
/// the view is far wider than the band's own scale, one when the view has zoomed to that
/// scale, and cubic growth after. Acuity is the allocator: that is why a 1 km cube of
/// ~3×10³¹ molecules is cheap, and why the page never needs a representative slab.
///
/// WHICH READING THIS COMPUTES, because the FSD admits two and they differ by nine orders.
/// The paragraph's parenthetical says the seed is where "a molecule at a pixel" first
/// resolves; at 1440 px that view is 432 nm across and holds 3.0e9 molecules, not one. The
/// reading that gives ONE is the view having zoomed to the BAND'S OWN SCALE — span ≈ ℓ —
/// and that reading also reproduces the paragraph's other number, "thousands at full
/// molecular zoom" (span = 10ℓ gives 1000). Both of the law's stated figures fall out of
/// it, so it is the one implemented here, and the parenthetical is the loose part.
function acuityPopulation(viewSpanM, lengthM) {
  if (!(viewSpanM > 0) || !(lengthM > 0) || viewSpanM < lengthM) return 0;
  return Math.floor(Math.pow(viewSpanM / lengthM, 3));
}

/// The view's span in metres, from the projection rather than from a guess: at camera
/// distance `d` the visible half-width on the plane through the origin is `d·tan(fov/2)`
/// in camera units, and one camera unit is one box span by construction (`sceneFrame`
/// normalises by `1/span`).
function viewSpanMetres(w) {
  // THE SCENE BOX'S OWN EXTENT, not a camera-derived quantity. This computed
  // `2·d·tan(fov/2)·boxSpan` from the camera distance until the two-box law landed — the
  // window-into-a-larger-scene model the operator explicitly rejected. Under the law the
  // scene box IS the view, so the span is world/zoom and nothing else, which also makes
  // the acuity law and the zoom the same number instead of two that could disagree.
  //
  // Disclosure: acuity figures shown before this change were computed the camera way and
  // do not compare with the ones shown now.
  const b = sceneBox(w);
  return 2 * Math.max(b.hx, b.hy, b.hz) * BOHR_TO_M;
}


/// The molecular band's live status: the de-allocation criterion, READ OUT.
///
/// No threshold is invented here. "A row reading ~0 is autonomous" needs a number that
/// separates ~0 from not-~0, and that number is the engine's to derive — from the grain
/// law's own stated bound between closure boundaries, which is where it belongs. Until it
/// exists this reports the DISTRIBUTION (best, worst) and the rows that are load-bearing
/// for a reason needing no threshold at all: the hand is on them.
function ladderStatus(w) {
  const rows = w.holon_row_count();
  if (rows === 0) return { rows: 0, text: "no composite in the scene yet" };
  let worst = 0;
  let best = Infinity;
  for (let k = 0; k < rows; k++) {
    const d = Math.abs(w.holon_row_closure_defect(k));
    if (d > worst) worst = d;
    if (d < best) best = d;
  }
  const held = w.holon_grabbed() >= 0 ? 1 : 0;
  return {
    rows,
    text: `${rows} row${rows === 1 ? "" : "s"} · defect ${fmtSci(best, 2)} … ${fmtSci(worst, 2)} Ha`
      + (held ? " · 1 held by the hand, load-bearing by construction" : ""),
  };
}

// ------------------------------------------- the ladder's readouts (FSD-W3 §11.2)
//
// The ladder rows above carry each band's STATE — live or fenced, with its owner, its exit
// and its citations. This table carries the NUMBERS those bands serve, and it exists as
// data for the same reason `NOT_SERVED` does: as markup it would be prose, and prose is
// where a provenance claim goes to rot.
//
// EVERY FIELD NAMES WHERE ITS DIGITS COME FROM, in one of exactly four forms, and `smoke.mjs`
// checks each of them against the artifact rather than taking the word for it:
//
//   live:<exports>          the digits are read this frame out of exports on
//                           REQUIRED_EXPORTS, which the boot refuses to run without.
//   export:<one export>     the digits are served by an export that is NOT YET IN THE
//                           ARTIFACT (WB-10.1 / WB-10.2 in build). Until it resolves the
//                           row renders FENCED — PENDING, names the export, and shows no
//                           digits. This is the whole of WB-7 applied to work in progress:
//                           the third option, a plausible number, does not exist.
//   declared:<file>#<field> a MEASURED INPUT the Hamiltonian never computes, read from a
//                           committed artifact and tagged DECLARED on the page (WB-1.7). A
//                           declared number presented as computed is the WB-7 lie in a new
//                           costume, so the tag is rendered beside the digits, not in a
//                           tooltip. If the file is not in the tree yet, the row pends.
//   computed:<inputs>       page arithmetic over inputs that are themselves live or
//                           declared, labelled as arithmetic. §11.2 grants exactly one of
//                           these to a fenced band — the cube's hydrostatic column — and
//                           the label says so on screen. Every input is named so the gate
//                           can resolve it; an unnamed input is how a constant walks in.
//
// The ids are the contract with the markup: `smoke.mjs` requires every id here to exist in
// index.html AND every `desc-` id in index.html to be here, so a row cannot be added to one
// without the other. A panel that silently stopped rendering would otherwise look exactly
// like a panel with nothing to say.
const DESCENT_FIELDS = [
  // --- the pick (WB-1.6)
  { id: "desc-pinned", label: "Pinned atom",
    source: "live:holon_grabbed, holon_atom_count, holon_atom_x, holon_atom_y, "
      + "holon_atom_z, holon_atom_species_z" },
  { id: "desc-membership", label: "In a molecule, or free",
    source: "export:holon_atom_in_molecule" },
  // --- the atom band (WB-10.2)
  { id: "desc-band-energy", label: "Electronic energy", source: "export:holon_atom_band_energy" },
  { id: "desc-band-electrons", label: "Electrons in the solve",
    source: "export:holon_atom_band_n_electrons" },
  { id: "desc-band-residual", label: "Solve residual", source: "export:holon_atom_band_residual" },
  { id: "desc-band-exit", label: "Solve exit", source: "export:holon_atom_band_exit" },
  // --- the nucleus band (WB-10.1 / WB-1.7)
  { id: "desc-nuc-z", label: "Z", source: "live:holon_atom_species_z" },
  { id: "desc-nuc-isotope", label: "Isotope",
    source: "declared:species_palette.json#isotope" },
  { id: "desc-nuc-mass", label: "Mass",
    // BOTH doors, and the tag does not change between them. The species table already
    // carries this measured input and is committed, so the row has honest digits today;
    // when `holon_nucleus_mass_u` lands the page reads the engine's copy instead. Either
    // way it is DECLARED — a measured input read back through an export is still a
    // measured input, and calling it LIVE because a function returned it would be the
    // costume WB-1.7 names.
    source: "declared:species_palette.json#mass_u",
    preferExport: "holon_nucleus_mass_u" },
  { id: "desc-nuc-spin", label: "Nuclear spin", source: "export:holon_nucleus_spin2" },
  { id: "desc-nuc-radius", label: "Charge radius",
    source: "export:holon_nucleus_charge_radius_fm" },
  { id: "desc-nuc-lambda", label: "Thermal de Broglie wavelength",
    // NOT computed here, deliberately. The page has the mass, the temperature and k_B and
    // could evaluate the closed form — and then there would be two implementations of one
    // number, which is how they start disagreeing. §11.4 gives it to the engine and gates
    // it there (`tests/nucleus.rs`: the wavelength reproduces the closed form on the
    // engine's own temperature readout), so the page waits for that door.
    source: "export:holon_nucleus_thermal_wavelength_bohr" },
  // --- the bit-identity gate (WB-10.2)
  { id: "desc-probe-wasm", label: "holon_law_probe() in this wasm",
    source: "export:holon_law_probe" },
  { id: "desc-probe-native", label: "the pinned native value",
    source: "declared:law_probe.json#energy_bits_hex" },
  { id: "desc-probe-verdict", label: "Bit identity",
    source: "computed:desc-probe-wasm, desc-probe-native" },
  // --- the cube band's hydrostatic column (§11.2, WB-2.4a)
  { id: "desc-density", label: "Scene mass density",
    source: "computed:holon_atom_count, holon_atom_species_z, holon_width, holon_height, "
      + "holon_depth, species_palette.json#mass_me" },
  { id: "desc-hydro-view", label: "ρ g h at the view's own depth",
    source: "computed:desc-density, holon_g_earth, holon_gravity_available, the scene "
      + "box's own vertical extent" },
  { id: "desc-hydro-km", label: "ρ g h at the cube band's 1 km",
    source: "computed:desc-density, holon_g_earth, holon_gravity_available, the cube "
      + "band's declared 1 km" },
];

/// SI, from the engine's box and the species table's declared masses. Both inputs are
/// measured — the box is the engine's own and the masses are the palette's — so this is
/// arithmetic, not a model, and the panel says so beside the digits.
///
/// It returns null rather than a number when the palette did not load: a density computed
/// with a missing mass would be a smaller number that looks like a measurement.
const ELECTRON_MASS_KG = 9.1093837015e-31;

function sceneDensitySI(w) {
  const n = w.holon_atom_count();
  if (n === 0 || PALETTE.size === 0) return null;
  let massMe = 0;
  for (let i = 0; i < n; i++) {
    const s = PALETTE.get(w.holon_atom_species_z(i));
    if (!s || !(s.mass_me > 0)) return null;
    massMe += s.mass_me;
  }
  const volumeM3 = w.holon_width() * w.holon_height() * w.holon_depth() * Math.pow(BOHR_TO_M, 3);
  if (!(volumeM3 > 0)) return null;
  return (massMe * ELECTRON_MASS_KG) / volumeM3;
}

/// g in SI, from the ENGINE's own constant rather than from a number in this file — the
/// same rule `applyControls` follows, so there is exactly one statement of what a G is.
/// Null where the field is refused: on a wrapping box there is no bottom to fall toward
/// (WB-2.4b), and a column that has no down has no ρ g h either.
function gravitySI(w) {
  if (w.holon_gravity_available() !== 1) return null;
  const gAu = Math.abs(w.holon_gravity());
  // a₀/aut² → m/s². One atomic unit of time is AU_TO_FS femtoseconds.
  const autS = AU_TO_FS * 1e-15;
  return (gAu * BOHR_TO_M) / (autS * autS);
}

// ---------------------------------------------------------------- formatting

function fmtEnergy(ha) {
  if (!Number.isFinite(ha)) return "—";
  const a = Math.abs(ha);
  if (a !== 0 && a < 1e-4) return `${ha.toExponential(3)} Ha`;
  return `${ha >= 0 ? "+" : ""}${ha.toFixed(6)} Ha`;
}

function fmtSci(x, digits = 3) {
  return Number.isFinite(x) ? x.toExponential(digits) : "—";
}

/// WB-1.4: real units, with the %-realtime figure beside them. The unit is chosen from
/// the magnitude rather than fixed, because this engine's honest rate on a laptop is
/// femtoseconds per second and a label pinned to "ps/s" would read 0.00 forever — which
/// is the same lie as the one WB-1.4 names, told in the other direction.
function fmtRate(fsPerSec) {
  if (!Number.isFinite(fsPerSec) || fsPerSec <= 0) return "—";
  if (fsPerSec >= 1e6) return `${(fsPerSec / 1e6).toFixed(2)} ns/s`;
  if (fsPerSec >= 1e3) return `${(fsPerSec / 1e3).toFixed(2)} ps/s`;
  return `${fsPerSec.toFixed(2)} fs/s`;
}

/// Metres, in whatever unit keeps the number readable. Separate from `fmtLength` because
/// that one takes bohr and this takes metres, and one function silently accepting both
/// is how a factor of 1.9e10 gets into a label.
function fmtMetres(m) {
  if (!Number.isFinite(m) || m <= 0) return "—";
  if (m < 1e-9) return `${(m * 1e12).toFixed(1)} pm`;
  if (m < 1e-6) return `${(m * 1e9).toFixed(2)} nm`;
  if (m < 1e-3) return `${(m * 1e6).toFixed(2)} µm`;
  if (m < 1) return `${(m * 1e3).toFixed(2)} mm`;
  if (m < 1e3) return `${m.toFixed(2)} m`;
  return `${(m / 1e3).toFixed(2)} km`;
}

/// Pascals, in whatever unit keeps the number readable, with the atmosphere beside it
/// because WB-2.4a's own statement of the exhibit is in atmospheres ("~9.8 MPa at 1 km,
/// about 97 atmospheres") and a reader should not have to convert to check the page against
/// the spec it cites.
function fmtPressure(pa) {
  if (!Number.isFinite(pa)) return "—";
  const atm = pa / 101325;
  const mag = Math.abs(pa) >= 1e6 ? `${(pa / 1e6).toFixed(3)} MPa`
    : Math.abs(pa) >= 1e3 ? `${(pa / 1e3).toFixed(3)} kPa`
      : Math.abs(pa) >= 1 ? `${pa.toFixed(3)} Pa`
        : `${pa.toExponential(3)} Pa`;
  return `${mag} (${Math.abs(atm) >= 0.01 ? atm.toFixed(2) : atm.toExponential(2)} atm)`;
}

function fmtLength(bohr) {
  const m = bohr * BOHR_TO_M;
  if (m < 1e-9) return `${(m * 1e12).toFixed(1)} pm`;
  if (m < 1e-6) return `${(m * 1e9).toFixed(3)} nm`;
  return `${(m * 1e6).toFixed(3)} µm`;
}

function tempIn(k) {
  if (State.tempUnit === "C") return `${(k - 273.15).toFixed(1)} °C`;
  if (State.tempUnit === "F") return `${((k - 273.15) * 9 / 5 + 32).toFixed(1)} °F`;
  return `${k.toFixed(1)} K`;
}

// ---------------------------------------------------------------- boot

async function boot() {
  // The artifact, and its digest. Fetched as bytes rather than streamed so the page can
  // hash exactly what it instantiates — `instantiateStreaming` would leave the manifest
  // describing bytes nobody here ever held.
  const response = await fetch("holon_render.wasm");
  if (!response.ok) throw new Error(`holon_render.wasm: HTTP ${response.status}`);
  const bytes = await response.arrayBuffer();
  State.artifact.bytes = bytes.byteLength;
  try {
    const digest = await crypto.subtle.digest("SHA-256", bytes);
    State.artifact.sha256 = [...new Uint8Array(digest)]
      .map((b) => b.toString(16).padStart(2, "0")).join("");
  } catch {
    // `crypto.subtle` is absent on insecure origins. The page still runs; the manifest
    // says the digest is unavailable rather than showing a number it did not compute.
    State.artifact.sha256 = "unavailable (crypto.subtle needs a secure origin)";
  }
  const { instance } = await WebAssembly.instantiate(bytes, {});
  const w = instance.exports;
  State.w = w;

  // FEATURE DETECTION against exports that actually exist. The atom viewer shipped for
  // months guarded on `holon_set_atom_z`, a name the engine has never exported, so the
  // guard was false for a reason unrelated to the capability it named. Every optional
  // path below is gated on the function it will actually call.
  const missing = REQUIRED_EXPORTS.filter((n) => typeof w[n] !== "function");
  if (missing.length) {
    throw new Error(`this wasm is not the workbench engine — missing ${missing.join(", ")}`);
  }

  await loadPalette();
  await loadLawProbe();

  // Three dimensions. The workbench is a 3D instrument; the 2D mid-plane is the atom
  // viewer's scene, not this one.
  w.holon_set_dims(1);
  // Closed walls: the box is the scene, and an open boundary would let the preset leave.
  w.holon_set_boundary(0);
  w.holon_set_census_enabled(1);

  calibrate(w);
  State.baseSimSpeed = w.holon_sim_speed();

  await loadPreset(State.mixture);

  // The unit constant, checked against the artifact rather than trusted, and checked HERE
  // rather than earlier: `holon_period` is derived from the loaded curve, so before a
  // preset exists it is zero and the ratio is NaN. Reading a unit out of an engine that
  // has no curve yet is asking a question the engine cannot have an answer to; the gate
  // caught this as a boot-time throw.
  const ratio = w.holon_period_fs() / w.holon_period();
  if (!(Math.abs(ratio - AU_TO_FS) < 1e-15)) {
    throw new Error(`time unit disagreement: engine ${ratio}, page ${AU_TO_FS}`);
  }

  State.booted = true;
  document.body.dataset.engine = "ready";
  requestAnimationFrame(frame);
}

/// Every export this file calls. Listed rather than discovered so that a wasm which is not
/// this engine is refused at boot with a name, instead of failing later as an undefined
/// call inside the frame loop. `smoke.mjs` reads this same list out of this file and
/// requires each one to resolve in the committed artifact, which is what keeps the list
/// honest as the page grows.
const REQUIRED_EXPORTS = [
  "holon_set_dims", "holon_set_boundary", "holon_set_census_enabled",
  "holon_table_generate", "holon_trimer_generate", "holon_trimer_loaded",
  "holon_trimer_nodes", "holon_trimer_peak",
  "holon_bank_clear", "holon_bank_register", "holon_bank_generate_pair", "holon_bank_pair_route",
  "holon_bank_pair_n_det", "holon_bank_pair_is_heavy", "holon_bank_slot",
  "holon_bank_filled_count", "holon_bank_browser_budget_seconds", "holon_bank_browser_knots",
  "holon_bank_set_browser_budget_seconds",
  "holon_bank_pair_predicted_seconds", "holon_bank_species_count",
  "holon_table_knots", "holon_table_r_e", "holon_table_d_e",
  "holon_chem_referee_residual", "holon_chem_referee_points",
  "holon_reset", "holon_rebase", "holon_atom_count", "holon_atom_x", "holon_atom_y",
  "holon_atom_z", "holon_atom_species_z", "holon_set_atom_species", "holon_atom_speed",
  "holon_width", "holon_height", "holon_depth", "holon_wall_inset",
  "holon_pair_count", "holon_pair_i", "holon_pair_j", "holon_pair_r", "holon_pair_bonded",
  "holon_bonded_count", "holon_cluster_count", "holon_cluster_atoms",
  "holon_e_kin", "holon_e_pair", "holon_e_three", "holon_e_wall", "holon_e_spring",
  "holon_w_ext", "holon_energy", "holon_ledger", "holon_ledger_origin",
  "holon_drift", "holon_drift_peak", "holon_drift_bound", "holon_energy_gate",
  "holon_momentum_residual", "holon_momentum_bound", "holon_momentum_gate",
  "holon_time", "holon_steps", "holon_temperature", "holon_frame",
  "holon_set_thermostat", "holon_thermostat_on",
  "holon_set_gravity", "holon_gravity", "holon_e_grav", "holon_g_earth",
  "holon_set_gravity_vec", "holon_gravity_x", "holon_gravity_y", "holon_gravity_z",
  "holon_gravity_available",
  "holon_box_scale", "holon_pressure", "holon_pressure_defined",
  "holon_advance_frame", "holon_step_frame", "holon_calibration_burst",
  "holon_set_calibration", "holon_substeps_per_second", "holon_n_max",
  "holon_sim_speed", "holon_set_sim_speed", "holon_dilation", "holon_rung",
  "holon_dt", "holon_period", "holon_period_fs", "holon_omega_dt",
  "holon_nearest_atom", "holon_grab", "holon_move_anchor_3d", "holon_release",
  "holon_grabbed", "holon_anchor_x", "holon_anchor_y", "holon_anchor_z",
  "holon_row_count", "holon_row_closure_defect", "holon_row_closure_defect_at_formation",
  "holon_row_kind", "holon_row_e_bond", "holon_row_member_count",
  "holon_census_molecules", "holon_census_formations", "holon_census_dissolutions",
  "holon_census_closure_rejections",
  "holon_fence_untabulated", "holon_water_loaded", "holon_trimer_surfaces",
  "holon_pairs_ready", "holon_bank_provenance_ok",
  // the doors shipped artifacts are pushed through (FSD-W3 §11.5): the pair bank's
  // node-wise door and the water door
  "holon_bank_table_begin", "holon_bank_table_knot", "holon_bank_table_knot_curvature",
  "holon_bank_table_finish", "holon_water_table_alloc", "holon_water_table_load",
  "holon_water_nodes", "holon_water_peak",
];

/// THE EXPORTS THE FINE BANDS ARE WAITING FOR — declared here and NOT in the list above,
/// which is a deliberate separation rather than an oversight.
///
/// `REQUIRED_EXPORTS` is a boot-time refusal: a wasm missing one of those names is not this
/// engine, and the page says so with the name instead of failing later inside the frame
/// loop. Putting a not-yet-built export in that list would take the whole page down for the
/// absence of a nucleus readout, which is the opposite of a fence.
///
/// These are the other kind of absence: WB-10.1 and WB-10.2 are in build in holon-render
/// right now, and until the rebuilt wasm ships them the atom and nucleus bands render
/// FENCED — PENDING with the missing export NAMED, and draw no digits at all. When a name
/// here starts resolving, `bandLiveness` flips its band with no edit to this file: the flip
/// is a property of the artifact, not of a word somebody changed.
///
/// `serves` and `what` are not decoration — `smoke.mjs` reads them, requires every name to
/// belong to a family FSD-W3 §11.4 actually commissions, and requires every band's
/// `liveWhen` to name only exports on this list.
const PENDING_EXPORTS = [
  { name: "holon_nucleus_spin2", serves: "nucleus", spec: "WB-10.1",
    what: "twice the nuclear spin of the most abundant isotope (DECLARED input)" },
  { name: "holon_nucleus_charge_radius_fm", serves: "nucleus", spec: "WB-10.1",
    what: "the nuclear charge radius in femtometres (DECLARED input)" },
  { name: "holon_nucleus_mass_u", serves: "nucleus", spec: "WB-10.1",
    what: "the isotope mass in unified atomic mass units (DECLARED input)" },
  { name: "holon_nucleus_thermal_wavelength_bohr", serves: "nucleus", spec: "WB-10.1",
    what: "the nucleus's thermal de Broglie wavelength at the scene's measured temperature "
      + "— COMPUTED in closed form by the engine, 0 where the temperature is undefined" },
  { name: "holon_atom_in_molecule", serves: "atom", spec: "WB-10.1",
    what: "0 if the atom is free, else 1 + the census row index it belongs to (WB-1.6)" },
  // THE SUB-ATOM BAND'S DOORS. `holon_hadron_solve` is the door — it runs an EXACT
  // colour-singlet solve and keeps it; the rest are read-backs of the stored sector, and
  // `holon_hadron_dim_for` / `holon_hadron_max_det` are the PRICE, readable before any
  // solve is asked for, so a zoom gesture is refused with a number instead of a hang.
  { name: "holon_hadron_solve", serves: "the fold below the atom", spec: "WB-10.7",
    what: "solve the exact colour-singlet sector of 1+1D QCD at (sites, coupling, baryon "
      + "number); 0 converged, 3 trivial, 5 bad parameters, 6 over the determinant cap" },
  { name: "holon_hadron_energy", serves: "the fold below the atom", spec: "WB-10.7",
    what: "the stored sector's exact ground-state energy, NaN if unsolved" },
  { name: "holon_hadron_occ", serves: "the fold below the atom", spec: "WB-10.7",
    what: "the quark density at one site, summed over colours — what the band draws" },
  { name: "holon_hadron_baryon_mass", serves: "the fold below the atom", spec: "WB-10.7",
    what: "(E1 - E0) / 2*sqrt(x) from the two stored sectors, NaN unless both are the same box" },
  { name: "holon_hadron_n_det", serves: "the fold below the atom", spec: "WB-10.7",
    what: "the stored sector's determinant count" },
  { name: "holon_hadron_exit", serves: "the fold below the atom", spec: "WB-10.7",
    what: "0 converged, 1 iteration cap, 2 stagnated, 3 trivial, 4 not solved" },
  { name: "holon_hadron_margin", serves: "the fold below the atom", spec: "WB-10.7",
    what: "min diag - E: non-negative for a true ground state, a defect on screen otherwise" },
  { name: "holon_hadron_dim_for", serves: "the fold below the atom", spec: "WB-10.7",
    what: "the determinant count a sector WOULD have, without solving it — the price" },
  { name: "holon_hadron_max_det", serves: "the fold below the atom", spec: "WB-10.7",
    what: "the cap above which the door refuses rather than attempts" },
  // THE DOOR, not a getter. The four rows below are READ-BACKS of the last solve; this is
  // what runs one. It is called on a pick change and on a throttle, never per frame — the
  // engine's own header says a molecule's FCI is milliseconds and its value changes with
  // every position, and a page that ran one every frame would be spending the scene's whole
  // budget on a readout.
  { name: "holon_atom_band_solve", serves: "atom", spec: "WB-10.2",
    what: "solve the picked atom, or the census molecule it belongs to, and keep the result "
      + "for the four getters; returns the exit code" },
  { name: "holon_atom_band_energy", serves: "atom", spec: "WB-10.2",
    what: "the picked atom's STO-3G FCI energy, solved on the lane engine in this page" },
  { name: "holon_atom_band_n_electrons", serves: "atom", spec: "WB-10.2",
    what: "how many electrons that solve carried" },
  { name: "holon_atom_band_residual", serves: "atom", spec: "WB-10.2",
    what: "the solve's residual" },
  { name: "holon_atom_band_exit", serves: "atom", spec: "WB-10.2",
    what: "how the solve ended: 0 converged, 1 iteration cap, 2 stagnated, 3 trivial, "
      + "4 not computed" },
  // THE BIT-IDENTITY PROBE, and its provenance is worth stating exactly: §11.4's WB-10.2
  // commissions the PROPERTY (wasm == native to the bit, pinned by `tests/wasm_law.rs`)
  // and names no export for it. The export name is WB-10.3's brief, agreed with the engine
  // lane. Recorded here rather than attributed to the FSD, because a citation to a line
  // that does not carry the claim is the failure this page's whole gate battery is about.
  { name: "holon_law_probe", serves: "atom", spec: "WB-10.3 brief (not named in §11.4)",
    what: "a fixed reference solve whose bits are pinned natively, so the page can display "
      + "EQUAL TO THE BIT rather than a tolerance" },
];

/// Is this export in the artifact this page instantiated? One place to ask, so a guard can
/// never name a function other than the one it guards — the atom viewer shipped for months
/// guarding on `holon_set_atom_z`, a name the engine has never exported.
function hasExport(name) {
  return !!State.w && typeof State.w[name] === "function";
}

/// THE FINE BANDS' FLIP RULE, as a pure function of the artifact.
///
/// A band whose quantities are served is LIVE; a band missing even one is FENCED — PENDING
/// and names what is missing. Both directions matter and only one of them is obvious: the
/// forward one stops the page drawing digits it does not have, and the reverse one stops
/// the band sitting fenced after the exports land, which is the absence-shaped rot the
/// gravity fence had before it was gated. Neither direction involves editing this file.
///
/// Pure, and deliberately so: `smoke.mjs` lifts this body out and runs it against a stub
/// artifact with and without the exports, which is a check that a second implementation in
/// the gate could not be.
function bandLiveness(liveWhen, has) {
  const names = liveWhen || [];
  const missing = names.filter((n) => !has(n));
  return { live: names.length > 0 && missing.length === 0, missing };
}

// ---------------------------------------------- the declared doors' sentinels (WB-10.1)
//
// A door that cannot serve a DECLARED value returns a sentinel rather than a plausible
// number — `u32::MAX` for the integer doors, `0.0` for the real ones (nucleus.rs's own
// header). The page fences on the sentinel, and these two readers are where that decision
// is made so it cannot be made differently in three places.
//
// THE ABI DETAIL THAT BITES. A wasm `u32` crosses into JavaScript through the i32 ABI, so
// `u32::MAX` arrives as **-1**, not as 4294967295. A guard written as `v === 4294967295`
// is false for the exact value it was written to catch, and the page would render "I = -1/2"
// for an element with no declared nucleus. `>>> 0` reinterprets the bits, which is the one
// operation that makes both spellings the same number.
//
// AND ZERO IS A REAL SPIN. ¹⁶O has spin 0 and ¹²C has spin 0, so `if (!spin2)` — the guard
// this would ordinarily be written as — fences the true value for two of the ten elements
// this page can draw. The test is the exact sentinel and nothing looser.
function declaredU32(v) {
  return (v >>> 0) === 0xFFFFFFFF ? null : (v >>> 0);
}

/// The real-valued doors' sentinel is `0.0`, and here zero is not a value any of them can
/// legitimately take: a charge radius of zero is a point nucleus and a mass of zero is not
/// an isotope. Negative and non-finite are refused for the same reason — a door that
/// returned one would be broken, and rendering it would put the breakage on screen as a
/// measurement.
function declaredPositive(v) {
  return Number.isFinite(v) && v > 0 ? v : null;
}

/// ONE READOUT ROW SERVED BY AN EXPORT, and the reason every such row goes through here
/// rather than writing its own guard.
///
/// Each of these rows had its own `hasExport(...) ? digits : fence` when this card was
/// first written, which is nine chances to get the guard right and one file in which a
/// missing `!` renders a number nobody computed. Routing them through one function makes
/// the discipline structural: `value` is a THUNK and it is not called at all when the
/// export is absent, so a pending row cannot evaluate — let alone display — a digit.
///
/// Pure, and `smoke.mjs` lifts it out and plants a throwing thunk against it: if the guard
/// is ever inverted the gate does not report a wrong string, it reports the throw.
/// TWO WAYS A ROW CAN HAVE NOTHING TO SHOW, and they are different facts about the world:
/// the DOOR is not in this artifact (the build has not landed), or the door is here and
/// SERVED ITS SENTINEL (nothing is declared for this element). The value thunk signals the
/// second by returning `null`, and both render as a fence with no digits — because a row
/// tagged DECLARED whose text reads FENCED is a panel contradicting itself, which is how
/// this was found.
function exportRow(name, has, kind, value, traceLive, tracePending) {
  if (!has(name)) {
    return { kind: "pending", text: `FENCED — PENDING ${name}`, trace: tracePending };
  }
  const v = value();
  if (v === null) {
    return {
      kind: "pending",
      text: `FENCED — ${name} serves its sentinel: nothing is declared for this element`,
      trace: traceLive,
    };
  }
  return { kind, text: v, trace: traceLive };
}

/// THE PICK (WB-1.6): which ONE atom the fine bands are about.
///
/// The hand's pick wins — an atom you are holding is the atom you are asking about. With no
/// hand on the scene the acuity law's seeding rule applies instead: pin ONE holon near the
/// view centre. "Near" is measured against the engine's own coordinates, which is reading
/// the scene rather than guessing at it; what may NOT be inferred from those coordinates is
/// whether the atom is in a molecule, and it is not — that comes from the census export and
/// from nothing else (WB-1.6 names the JavaScript distance heuristic as the thing forbidden).
function pinnedAtomIndex(w) {
  const grabbed = w.holon_grabbed();
  if (grabbed >= 0) return { index: grabbed, how: "the hand's pick" };
  const n = w.holon_atom_count();
  if (n === 0) return { index: -1, how: "no atom in the scene" };
  const b = sceneBox(w);
  let best = -1;
  let bestD = Infinity;
  for (let i = 0; i < n; i++) {
    const dx = w.holon_atom_x(i) - b.cx;
    const dy = w.holon_atom_y(i) - b.cy;
    const dz = w.holon_atom_z(i) - b.cz;
    const d = dx * dx + dy * dy + dz * dz;
    if (d < bestD) { bestD = d; best = i; }
  }
  return { index: best, how: "the acuity law's seed — nearest the view centre" };
}

/// Is the observer AT the fine bands — has the descent been entered?
///
/// TWO DOORS, because §11.2 opens two and they are not the same. WB-1.6 says the fine bands
/// are reached through a PICKED atom, so a hand on an atom is a descent at any zoom: you are
/// asking about that atom and the page should answer. With no hand, the acuity law's own
/// rule applies — the molecular band's population falls to zero exactly when the view span
/// drops below that band's scale, which is the moment §9c calls "the next tier starts to
/// matter". No third threshold is invented here; a third could disagree with these two.
///
/// This governs the LABEL, never whether the numbers are real. The pinned atom exists at
/// every zoom and every readout about it traces the same way; what the descent changes is
/// whether the observer has arrived, and the page says which rather than hiding the rows.
function descentActive(w) {
  if (w.holon_grabbed() >= 0) return true;
  const molecular = LADDER.find((b) => b.band === "molecular");
  return acuityPopulation(viewSpanMetres(w), molecular.lengthM) === 0;
}

/// THE ATOM BAND'S SOLVE, run on a pick change and on a throttle — never per frame.
///
/// The engine's own door says why: a molecule's FCI is milliseconds and its value changes
/// with every position, so a per-frame solve would spend the scene's whole budget on a
/// readout and WB-6.2 would be paying for it in dilated time. What the page owes instead is
/// HONESTY ABOUT STALENESS, which is why the solve's frame is recorded and displayed: a
/// number that was true 400 frames ago is not wrong, but a page that does not say when it
/// was taken is inviting it to be read as now.
const BAND_SOLVE_INTERVAL_MS = 500;

function maybeSolveAtomBand(w, atom) {
  if (atom < 0 || !hasExport("holon_atom_band_solve")) return;
  const now = performance.now();
  const s = State.atomBand;
  if (s.atom === atom && now - s.atMs < BAND_SOLVE_INTERVAL_MS) return;
  s.exit = w.holon_atom_band_solve(atom);
  s.atom = atom;
  s.atMs = now;
  s.atFrame = w.holon_frame();
  s.atTimeFs = w.holon_time() * AU_TO_FS;
}

async function loadPalette() {
  try {
    const r = await fetch("species_palette.json");
    const d = await r.json();
    for (const s of d.species) PALETTE.set(s.Z, s);
  } catch {
    // The palette is DRAWING data, not physics. Losing it costs colour and proportion,
    // not correctness, so the page continues and says so in the manifest.
    PALETTE = new Map();
  }
}

/// THE NATIVE REFEREE'S PINNED VALUE (WB-10.2), fetched rather than embedded.
///
/// `law_probe.json` is written by the engine lane out of `tests/wasm_law.rs`, which
/// computes the reference solve natively and pins its bits. The page displays the wasm's
/// own `holon_law_probe()` beside it and says EQUAL TO THE BIT or shows the mismatch — the
/// claim §11.1 makes possible, that a solve in the shipped wasm is the same arithmetic as
/// the native referee rather than merely close to it.
///
/// ABSENT IS NOT ZERO AND NOT GREEN. The file is not in the tree yet; a missing referee
/// means the row PENDS. A bit-identity row that went green because there was nothing to
/// disagree with would be the vacuous-success shape with a checkmark on it.
async function loadLawProbe() {
  try {
    const r = await fetch("law_probe.json");
    if (!r.ok) throw new Error(`HTTP ${r.status}`);
    const d = await r.json();
    if (typeof d.energy_bits_hex !== "string" || !/^[0-9a-fA-F]{16}$/.test(d.energy_bits_hex)) {
      throw new Error("energy_bits_hex is not 16 hex characters");
    }
    State.lawProbe = d;
  } catch (e) {
    State.lawProbe = null;
    State.lawProbeWhy = String(e && e.message ? e.message : e);
  }
}

/// The raw f64 bits of a double, as the 16 lowercase hex characters `law_probe.json` pins.
/// Big-endian, because that is the order the digits are written in and a comparison of two
/// strings written in opposite orders is a comparison that always fails.
function f64Bits(x) {
  const buf = new ArrayBuffer(8);
  new DataView(buf).setFloat64(0, x, false);
  return [...new Uint8Array(buf)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

/// EQUAL TO THE BIT, and the comparison is on the bits — there is no tolerance anywhere in
/// this function and there must not be. A near-equality here would pass on two numbers that
/// differ, which is the exact property §11.1 says the lane kernel now makes checkable and
/// the exact property a tolerance would throw away.
function lawProbeVerdict(wasmBits, nativeBits) {
  if (!wasmBits || !nativeBits) return null;
  return wasmBits.toLowerCase() === nativeBits.toLowerCase();
}

/// Measure THIS device rather than assuming it (M-IDLE-CALIBRATED-TIMEOUT / WB-2.3).
///
/// "How fast is the browser" is not answerable from a developer's machine, so the page
/// finds out on load: a burst of pure physics, no rendering, timed on this side because
/// `std::time` does not exist on wasm32-unknown-unknown. The burst is sized in TIME so a
/// slow device is not punished with a long stall.
function calibrate(w) {
  const targetMs = 200;
  let substeps = 2000;
  let elapsed = 0;
  let total = 0;
  // Discard the first burst: it pays for JIT warm-up the steady rate should not carry.
  w.holon_calibration_burst(500);
  const t0 = performance.now();
  while (performance.now() - t0 < targetMs) {
    const a = performance.now();
    w.holon_calibration_burst(substeps);
    const b = performance.now();
    elapsed += b - a;
    total += substeps;
    if (b - a < 20) substeps *= 2;
  }
  if (elapsed > 0) w.holon_set_calibration((total / elapsed) * 1000);
}

// ---------------------------------------------------------------- presets (WB-3.1)

/// Reset the scene to a preset, and MEASURE what the engine will serve for it.
///
/// Every curve is requested and the engine's answer is recorded verbatim, including its
/// refusal code. Nothing here decides in advance what is available: the (O,O) refusal
/// below is the engine's own in-browser split talking, not a rule this file keeps.
/// Yield to the compositor so a stage line actually paints before the next blocking
/// wasm call. `requestAnimationFrame` alone is not enough — the frame has to be allowed to
/// composite — so this waits for the frame AND a macrotask after it.
function paint(message) {
  if (message !== undefined) put("boot-stage", message);
  return new Promise((resolve) => requestAnimationFrame(() => setTimeout(resolve, 0)));
}

/// Prices this page has actually PAID, keyed by pair. Measured wall-clock milliseconds, so
/// the second time a user meets a curve the cost is a measurement rather than a warning.
const PAID_PRICES = new Map();
const priceKey = (za, zb) => `${Math.min(za, zb)}-${Math.max(za, zb)}`;

// ---------------------------------------------------------------- shipped artifacts
//
// Curves and surfaces the browser cannot afford to solve at load arrive as committed text
// artifacts and are PUSHED through the engine's own doors: the pair bank's node-wise door
// (`holon_bank_table_*`, whose provenance gate weighs the declared uncertainty and refuses
// a file on the wrong side of the split) and the water door (`holon_water_table_alloc` /
// `holon_water_table_load`, whose reader is the native parser to the bit —
// tests/water_door.rs). Each artifact is pinned by the SHA-256 of its bytes, computed here
// over what was fetched; a mismatch is a fence, not a warning, because a table that is not
// the one the tests certified is not a table this page may serve. `smoke.mjs` checks every
// pin against the file in the tree, so a re-emitted artifact cannot ship under a stale pin.
const SHIPPED = {
  pairs: {
    "8,1": { file: "tables/HO.json",
      sha256: "0dc5dd77a0cfc58891993601d9c429aefb66fbed6b14cec77ef79891049905df" },
    // THE CURVE THE WATER SCENE COULD NOT STEP WITHOUT. 2,025 determinants over 192 knots,
    // 1,785 s on the mesh — minutes, not a page load, in any browser — so it ships.
    "8,8": { file: "tables/O2.json",
      sha256: "b5c4802cd4968e76e3f6aac077e724734bb670e25468ab6400093828e4a80121" },
  },
  water: { file: "tables/s2_water_table.txt", nodes: 105105,
    sha256: "9cb10675aaafe3d0a98486befb506165193a07a7f9a85ade87d75dbd1804a681" },
};

/// Fetch a pinned artifact and refuse it unless its bytes digest to the pin.
async function fetchPinned(entry) {
  let res;
  try { res = await fetch(entry.file); } catch (e) { return { ok: false, why: `${entry.file}: ${e.message}` }; }
  if (!res.ok) return { ok: false, why: `${entry.file}: HTTP ${res.status}` };
  const bytes = new Uint8Array(await res.arrayBuffer());
  let sha;
  try {
    sha = [...new Uint8Array(await crypto.subtle.digest("SHA-256", bytes))]
      .map((b) => b.toString(16).padStart(2, "0")).join("");
  } catch {
    return { ok: false, why: `${entry.file}: crypto.subtle is unavailable on this origin, so `
      + "the artifact cannot be pinned and is not served" };
  }
  if (sha !== entry.sha256) {
    return { ok: false, why: `${entry.file}: SHA-256 ${sha.slice(0, 12)}… is not the pinned `
      + `${entry.sha256.slice(0, 12)}… — the file in the tree is not the one this page certifies` };
  }
  return { ok: true, bytes, sha };
}

/// Push a shipped pair curve (the `emit_pair_tables` JSON) through the bank's node-wise
/// door. The return is the engine's: 1 when the bank committed the slot, else its refusal
/// code with the reason spelled out.
function pushShippedPair(w, za, zb, j) {
  const slot = w.holon_bank_slot(za, zb);
  if (slot < 0) return { code: 0, why: `${sym(za)}–${sym(zb)} is not a registered pair in the bank` };
  const n = j.R_grid_bohr.length;
  if (w.holon_bank_table_begin(slot, n) !== 1) return { code: 0, why: `the bank refused a ${n}-knot grid` };
  for (let i = 0; i < n; i++) {
    if (w.holon_bank_table_knot(slot, i, j.R_grid_bohr[i], j.E_hartree[i], j.F_hartree_per_bohr[i]) !== 1
      || w.holon_bank_table_knot_curvature(slot, i, j.E2_hartree_per_bohr2[i]) !== 1) {
      return { code: 0, why: `knot ${i} of ${n} was refused by the bank` };
    }
  }
  const route = j.solver_route === "determinant" ? 1 : j.solver_route === "DMRG" ? 2 : 0;
  const code = w.holon_bank_table_finish(slot, j.R_e, j.D_e, j.E_asymptote, route,
    j.species.n_determinants, j.species.n_basis, j.uncertainty_hartree, j.exact_in_model ? 1 : 0);
  if (code === 1) return { code, knots: n };
  const why = code === 21
    ? "REFUSED by the bank's split (Refusal::SplitViolated): the engine's cost model puts "
      + "this pair on the SOLVE-HERE side, and a shipped file for a pair the browser is "
      + "expected to solve itself is refused, not welcomed. The model and the file "
      + "disagree; the model is the engine's, and this page reports it."
    : refusalText(code, j.species.n_determinants);
  return { code, why, knots: n };
}

/// Push the (O,H,H) table's bytes through the water door. 1 when the engine's parser read
/// them as this build's table.
function pushWater(w, bytes) {
  const ptr = w.holon_water_table_alloc(bytes.length);
  // the buffer is read AFTER the reservation: a reservation can grow linear memory, and a
  // view taken before it would be detached
  new Uint8Array(w.memory.buffer, ptr, bytes.length).set(bytes);
  return w.holon_water_table_load();
}

async function loadPreset(key, { pay = null } = {}) {
  const w = State.w;
  const preset = PRESETS[key];
  State.mixture = key;
  UI["boot-overlay"]?.classList.remove("hidden");

  // A fresh bank. Switching composition must not leave the previous preset's curves in
  // the slots, where they would silently serve the wrong pair.
  // The load budget is THIS PAGE's patience, declared here so the engine's split is
  // enforced against a number the page owns: five seconds of main thread per curve at
  // load. Anything dearer is refused before it is computed and must be shipped instead.
  w.holon_bank_set_browser_budget_seconds(5);
  w.holon_bank_clear();

  const served = {
    label: preset.label,
    pairs: [],
    trimer: { state: "none", detail: "" },
    stepsAllowed: false,
    fences: [],
    priced: [],
  };

  for (const z of preset.species) w.holon_bank_register(z);

  // H-H through the CHEAP DOOR. This also gives the clocks a derived dt before anything
  // steps, which is why it runs for every preset including the ones with no hydrogen in
  // the scene.
  await paint("solving the H–H curve (STO-3G full CI)…");
  const t0 = performance.now();
  const hhCode = w.holon_table_generate(0.6, 12.0, 192);
  const hhMs = performance.now() - t0;
  PAID_PRICES.set(priceKey(1, 1), hhMs);
  served.pairs.push({
    za: 1, zb: 1, route: 1, nDet: w.holon_bank_pair_n_det(1, 1),
    heavy: false, code: hhCode, ok: hhCode === 1, ms: hhMs, door: "stream_table",
  });

  for (const [za, zb] of preset.pairs) {
    const route = w.holon_bank_pair_route(za, zb);
    const nDet = w.holon_bank_pair_n_det(za, zb);
    const heavy = w.holon_bank_pair_is_heavy(za, zb) === 1;
    const shipped = SHIPPED.pairs[`${za},${zb}`];

    // A SHIPPED curve comes first, whatever the split says about solving it here: the
    // bank's own provenance gate decides whether the file is admitted (it weighs the
    // declared uncertainty, and it refuses a file for a pair the browser is expected to
    // solve itself), and its verdict is the engine's. The bytes are pinned before a knot
    // is pushed.
    if (shipped) {
      await paint(`loading the shipped ${sym(za)}–${sym(zb)} curve (${shipped.file})…`);
      const got = await fetchPinned(shipped);
      let r = { code: 0, why: got.why };
      if (got.ok) {
        let table = null;
        try { table = JSON.parse(new TextDecoder().decode(got.bytes)); } catch { table = null; }
        r = table ? pushShippedPair(w, za, zb, table) : { code: 0, why: `${shipped.file} is not JSON` };
      }
      served.pairs.push({ za, zb, route, nDet, heavy, code: r.code, ok: r.code === 1,
        door: "shipped", knots: r.knots, sha: got.sha });
      if (r.code !== 1) {
        served.fences.push({ ...FENCE_REGISTER.artifactRefused,
          what: `${sym(za)}–${sym(zb)} pair curve — shipped artifact`, why: r.why });
      }
      continue;
    }

    // A pair past the engine's own in-browser split is refused BEFORE it is computed, and
    // the refusal is instant — so it costs nothing to ask and the answer is the engine's,
    // not a rule this file keeps.
    if (heavy) {
      const code = w.holon_bank_generate_pair(za, zb, w.holon_bank_browser_knots());
      served.pairs.push({ za, zb, route, nDet, heavy, code, ok: false, door: "refused" });
      served.fences.push({ what: `${sym(za)}–${sym(zb)} pair curve`, why: refusalText(code, nDet),
        ...FENCE_REGISTER.splitViolated });
      continue;
    }

    // Within the split, and therefore permitted — but permitted is not free. The engine's
    // limit is a determinant count and the wall-clock cost varies by two orders of
    // magnitude underneath it, so this page will not spend an unbounded amount of a main
    // thread on the user's behalf. The curve is offered at its declared cost and solved
    // when asked for.
    const paidBefore = PAID_PRICES.get(priceKey(za, zb));
    if (pay !== priceKey(za, zb) && paidBefore === undefined) {
      served.priced.push({ za, zb, nDet, key: priceKey(za, zb) });
      served.pairs.push({ za, zb, route, nDet, heavy, code: 0, ok: false, door: "priced" });
      served.fences.push({
        ...FENCE_REGISTER.unpaidCurve,
        what: `${sym(za)}–${sym(zb)} pair curve — UNPAID`,
        why: `the engine permits this solve (${nDet.toExponential(2)} determinants, predicted `
          + `${w.holon_bank_pair_predicted_seconds(za, zb).toFixed(1)} s against the page's `
          + `${w.holon_bank_browser_budget_seconds().toFixed(0)} s load budget) but it is not `
          + "free: on the development machine it took about fifteen seconds of the main "
          + "thread. It is offered rather than spent, and the price you pay is measured "
          + "and reported. Press SOLVE in the mixture panel to buy it.",
      });
      continue;
    }

    await paint(`solving the ${sym(za)}–${sym(zb)} curve — ${nDet.toExponential(2)} determinants, this will block…`);
    const a = performance.now();
    const code = w.holon_bank_generate_pair(za, zb, w.holon_bank_browser_knots());
    const ms = performance.now() - a;
    if (code === 1) PAID_PRICES.set(priceKey(za, zb), ms);
    served.pairs.push({ za, zb, route, nDet, heavy, code, ok: code === 1, ms, door: "generate_pair" });
    if (code !== 1) {
      served.fences.push({ what: `${sym(za)}–${sym(zb)} pair curve`, why: refusalText(code, nDet),
        ...FENCE_REGISTER.splitViolated });
    }
  }

  // The (O,H,H) surface, SHIPPED and pushed through the water door for every preset whose
  // scene can form the triple. The door's reader is the native parser to the bit
  // (tests/water_door.rs), so what the page serves here is the table the S2 campaign
  // certified — or nothing, with the reason named.
  served.water = { state: "not needed", detail: "no (O,H,H) triple can form in this scene" };
  if (preset.species.includes(8) && preset.species.includes(1)) {
    await paint(`loading the shipped (O,H,H) three-body table (${SHIPPED.water.nodes.toLocaleString()} nodes)…`);
    const got = await fetchPinned(SHIPPED.water);
    if (!got.ok) {
      served.water = { state: "refused", detail: got.why };
    } else {
      const code = pushWater(w, got.bytes);
      served.water = code === 1 && w.holon_water_loaded() === 1
        ? { state: "served",
          detail: `${SHIPPED.water.nodes.toLocaleString()} solved nodes filling a `
            + `${w.holon_water_nodes().toLocaleString()}-node symmetric grid, peak |dE₃| `
            + `${fmtSci(w.holon_water_peak())} Ha, pushed through holon_water_table_load and `
            + `read by the engine's own parser (SHA-256 ${got.sha.slice(0, 12)}…)` }
        : { state: "refused",
          detail: "the engine's parser refused the artifact: not this build's grid rule" };
    }
    if (served.water.state !== "served") {
      served.fences.push({ ...FENCE_REGISTER.artifactRefused,
        what: "(O,H,H) three-body surface — shipped artifact", why: served.water.detail });
    }
  }

  // The three-body sector. The homonuclear H3 surface generates in the browser — nine
  // determinants a node over 14,157 nodes — so it is loaded for every preset that can
  // meet an H-H-H triple, and its presence is also what keeps the fence counter honest.
  await paint(`generating the H₃ three-body surface (14,157 nodes)…`);
  const t = w.holon_trimer_generate();
  const loaded = t === 1 && w.holon_trimer_loaded() === 1;
  const fencedTriples = untabulatedTriples(preset, served.water.state === "served");
  served.fencedTriples = fencedTriples.join(", ");
  const waterLine = served.water.state === "served"
    ? `(O,H,H) is SERVED from the shipped table (${SHIPPED.water.nodes.toLocaleString()} solved `
      + "nodes) pushed through the water door. "
    : "";
  if (!loaded) {
    served.trimer = { state: "refused", detail: "the H₃ generator declined this grid." };
    served.fences.push({ what: "H₃ three-body surface", why: served.trimer.detail,
      ...FENCE_REGISTER.trimerRefused });
  } else if (fencedTriples.length) {
    served.trimer = {
      state: "partly served",
      detail:
        `(H,H,H) is SERVED from the engine's own ${w.holon_trimer_nodes()}-node surface, `
        + `generated in the browser at load, peak |dE₃| ${fmtSci(w.holon_trimer_peak())} Ha. `
        + waterLine
        + `${served.fencedTriples} ${fencedTriples.length === 1 ? "is" : "are"} FENCED: no `
        + "surface for them is tabulated in this build — (O,O,H) and (O,O,O) are not computed "
        + "anywhere yet. Every such encounter is refused and counted below rather than "
        + "interpolated across or zeroed.",
    };
    served.fences.push({
      ...FENCE_REGISTER.untabulatedTriples,
      what: `${served.fencedTriples} three-body surface${fencedTriples.length === 1 ? "" : "s"}`,
      why: "not tabulated in this build; the encounters are refused and counted, and the "
        + "count is a live readout.",
    });
  } else {
    served.trimer = {
      state: "served",
      detail: `${w.holon_trimer_nodes()} nodes, generated in the browser at load, `
        + `peak |dE₃| ${fmtSci(w.holon_trimer_peak())} Ha. ${waterLine}Every triple this `
        + "scene can form is on a certified surface.",
    };
  }

  // The scene itself, in TWO resets, which is not a redundancy.
  //
  // `holon_reset` both places the atoms AND derives the opener's expansion speed from the
  // curves the atoms will actually meet each other on — the guarantee that the scene opens
  // handing out no bonds nobody paid for. Species can only be assigned to atoms that
  // exist, so the first reset establishes how many there are, the assignment says what
  // they ARE, and the second reset re-derives the opener against the RIGHT curves. Doing
  // it in one pass would open an O:2H box on hydrogen's well depth.
  w.holon_reset(State.atomsRequested);
  State.atomsActual = w.holon_atom_count();
  applyComposition(preset, State.atomsActual);
  w.holon_reset(State.atomsRequested);
  applyComposition(preset, w.holon_atom_count());
  // A changed composition is a changed scene, so the ledger's origin moves with it;
  // comparing against an origin taken before this composition existed would report a drift
  // no integrator produced.
  w.holon_rebase();

  // MAY THIS SCENE STEP AT ALL — asked AFTER the composition, which is the whole point.
  //
  // `holon_pairs_ready` is a question about the pairs THIS SCENE'S ATOMS can meet, so on a
  // box that is still all hydrogen it answers about hydrogen. Asked before the composition
  // was applied it returned 1 for an O:2H scene whose (O,O) slot is empty and which cannot
  // take a single step — and the page then reported a frozen box as "SETTLING · 0.0 K",
  // which is the vacuous-success shape exactly. Measured either way: 1 before the
  // composition, 0 after.
  served.stepsAllowed = w.holon_pairs_ready() === 1;
  if (!served.stepsAllowed) {
    served.fences.push({
      ...FENCE_REGISTER.cannotStep,
      what: "dynamics — this scene cannot step",
      why: "a curve this scene's atoms can meet each other on is missing, so the engine "
        + "refuses to integrate. Nothing is drawn moving because nothing is moving, and "
        + "the temperature reads zero because the opener cannot derive a velocity against "
        + "a curve that is not there.",
    });
  }

  State.served = served;
  applyControls();
  State.clockWindow = { t0: null, simFs0: 0, frames: 0 };
  State.replay = { last: null, prev: null, matched: null };
  // the reset re-derived the box, so the hand's scale starts over with it
  State.boxScale = 1.0;
  State.lastScaleRefusal = null;
  syncSizeSlider();
  renderStatics();
  renderSizeAxis();
  UI["boot-overlay"]?.classList.add("hidden");
}

/// Stamp the preset's composition onto the atoms that exist.
///
/// Unconditional for every preset that is not pure hydrogen — including the SINGLE-species
/// ones. `Sim::empty` seeds hydrogen, so a pure-O preset that skipped this would be a box
/// of hydrogen wearing an oxygen label, which is the exact species/curve disagreement the
/// engine's plant (i) is about.
function applyComposition(preset, n) {
  const w = State.w;
  if (preset.species.length === 1 && preset.species[0] === 1) return;
  const order = preset.composition === "single-o"
    ? Array.from({ length: n }, (_, i) => (i === 0 ? 8 : 1))
    : compositionOrder(preset.species, n);
  for (let i = 0; i < n; i++) w.holon_set_atom_species(i, order[i]);
}

/// O:2H means one oxygen per two hydrogens, laid down in that ratio. Deterministic, so
/// WB-5.4's replay claim covers composition as well as coordinates.
function compositionOrder(species, n) {
  if (species.length === 2 && species[0] === 1 && species[1] === 8) {
    const out = [];
    for (let i = 0; i < n; i++) out.push(i % 3 === 0 ? 8 : 1);
    return out;
  }
  const out = [];
  for (let i = 0; i < n; i++) out.push(species[i % species.length]);
  return out;
}

const SYMBOLS = { 1: "H", 8: "O" };
const sym = (z) => SYMBOLS[z] || `Z=${z}`;

/// Turn a `LoadStatus`-space code back into the engine's own reason.
///
/// The codes are the engine's, not this file's: `GENERATOR_REFUSED = 6`,
/// `PROVENANCE_REFUSED = 16` with the reason carried in the offset, `CURVE_INFEASIBLE`
/// and `BANK_FULL` above them. Reproduced here so the page can NAME a refusal instead of
/// printing an integer, and kept narrow — an unrecognised code is reported as itself
/// rather than guessed at.
function refusalText(code, nDet) {
  if (code === 21) {
    return "REFUSED by the engine's in-browser split (Refusal::SplitViolated): this "
      + `curve is ${nDet.toExponential(3)} determinants and its predicted load cost exceeds `
      + `the page's ${State.w.holon_bank_browser_budget_seconds().toFixed(0)} s budget `
      + "(a declared horizon the host can raise, not a cap). It is a mesh job, not a "
      + "page-load job, and the engine declines BEFORE spending the time rather than after.";
  }
  if (code === 6) return "REFUSED: the grid request was not a grid (GENERATOR_REFUSED).";
  if (code >= 16 && code <= 24) return `REFUSED at the provenance door, reason ${code - 16}.`;
  return `REFUSED with engine code ${code}.`;
}

// ---------------------------------------------------------------- controls (WB-2)

function applyControls() {
  const w = State.w;
  if (!w) return;
  w.holon_set_thermostat(State.thermostatOn ? 1 : 0, State.targetK);
  // Gravity as a WORLD VECTOR (WB-2.4c). The magnitude is converted through the ENGINE's
  // own constant, so there is exactly one statement of what a G is and it is not in this
  // file; the direction comes from the box's tilt. A refusal (code 80) is reachable now
  // that boundary mode 2 selects Periodic, so the panel reads `holon_gravity_available`
  // rather than assuming.
  const g = State.gravityG * w.holon_g_earth();
  const th = (State.tiltDeg * Math.PI) / 180;
  // Tilting the box by +θ tips the world's "down" toward +x in BOX coordinates. The
  // return code is READ, not discarded: on a wrapping box this refuses, and a page that
  // ignored the refusal would leave a gravity slider that appears to work and does not.
  const gcode = w.holon_set_gravity_vec(g * Math.sin(th), -g * Math.cos(th), 0);
  State.gravityRefused = gcode !== 1;
  // The governor moves the SIM-SPEED, never dt. `holon_set_allow_dt_growth` is left off,
  // so the engine holds exactness and delivers any shortfall as honest time dilation
  // (WB-6.2: no reduced-accuracy mode, only slower time).
  w.holon_set_sim_speed(State.baseSimSpeed * State.govBias);
}

// ---------------------------------------------------------------- frame loop

function frame(now) {
  const w = State.w;

  // Clock 2, MEASURED. The first frame has no predecessor to measure against, so it
  // advances nothing rather than guessing an interval.
  const wallDt = State.lastFrameMs == null ? 0 : (now - State.lastFrameMs) / 1000;
  State.lastFrameMs = now;

  if (!State.paused && State.served && State.served.stepsAllowed && wallDt > 0) {
    w.holon_advance_frame(Math.min(wallDt, 0.25));
  }

  measureRate(now);
  render3D();
  renderTelemetry();
  requestAnimationFrame(frame);
}

/// WB-1.4, measured. The delivered rate is simulated time actually integrated per wall
/// second — `holon_time()` differenced over a one-second window — not the sim-speed that
/// was REQUESTED. The two differ whenever the governor dilates, which is exactly the case
/// the honest readout exists for.
function measureRate(now) {
  const w = State.w;
  const cw = State.clockWindow;
  const simFs = w.holon_time() * AU_TO_FS;
  if (cw.t0 == null) {
    State.clockWindow = { t0: now, simFs0: simFs, frames: 0 };
    return;
  }
  cw.frames += 1;
  const wall = (now - cw.t0) / 1000;
  if (wall >= 0.5) {
    const fsPerSec = (simFs - cw.simFs0) / wall;
    State.rate.fsPerSec = fsPerSec;
    // femtoseconds of simulated time per second of wall time, as a percentage.
    State.rate.pctRealtime = 100 * fsPerSec * 1e-15;
    State.rate.fps = cw.frames / wall;
    State.clockWindow = { t0: now, simFs0: simFs, frames: 0 };
  }
}

// ---------------------------------------------------------------- the scene box
//
// THE TWO-BOX LAW (FSD-W2). Four WORLD boxes carry the physics, one SCENE box carries the
// view, and they are never the same knob:
//
//   * the WORLD box is the engine's own box. The hand acts on it through `holon_box_scale`
//     — that is the pressure control — and every whole-only observable (the virial
//     pressure, the temperature, the census's phase fractions) is computed over ALL of it.
//   * the SCENE box is world / zoom. Holons outside it are removed FROM THE VIEW across
//     all six faces. Nothing leaves the simulation.
//
// WHY NOTHING LEAVES, stated because the alternative was measured and is wrong: zooming by
// scaling the world box is AFFINE, and affine scale multiplies density by 1/f³. Three
// halvings measured a density ratio of 512.0 — a "zoom" that compressed the water
// 512-fold every three steps would be showing a different substance at every scale, which
// is the opposite of a scale ladder. So the zoom touches the quotient and never the Sim,
// and `zoom_leaves_world_density_invariant` in the gate plants the wrong door to prove it.
//
// AND THE WORLD BOX IS THE RESERVOIR. Zoom-out does not thaw a frozen cache and does not
// synthesise new matter: the same holons come back at their CURRENT evolved state, because
// the world kept simulating them while they were unwatched. A frozen reservoir would
// return you to a world that stopped when you looked away, which is its own fake.

/// The scene box's half-extents in bohr: the world box divided by the zoom ratio.
function sceneBox(w) {
  const z = Math.max(1, State.zoom);
  const hx = w.holon_width() / (2 * z);
  const hy = w.holon_height() / (2 * z);
  const hz = w.holon_depth() / (2 * z);
  // The centre is the CAMERA TARGET, defaulting to the world-box centre until aimed.
  const t = State.camera.target
    ?? { x: 0.5 * w.holon_width(), y: 0.5 * w.holon_height(), z: 0.5 * w.holon_depth() };
  // CLAMPED to stay wholly inside the world box: near a wall the scene box SLIDES rather
  // than protrudes. The quotient sets its size and the clamp sets its position, so a view
  // aimed at a corner still shows a full box of water rather than a box half full of
  // nothing that is not even vacuum — it would be outside the domain entirely.
  const clamp = (v, h, extent) => Math.min(Math.max(v, h), extent - h);
  return {
    hx, hy, hz,
    cx: clamp(t.x, hx, w.holon_width()),
    cy: clamp(t.y, hy, w.holon_height()),
    cz: clamp(t.z, hz, w.holon_depth()),
  };
}


/// Which holons the scene box contains, and what crossed a face since the last frame.
///
/// Membership is DATA — a predicate on position — which is why the scene-box cut needs no
/// engine door at all: the producer of the draw list simply does not emit what is outside,
/// and nothing in the Sim moves. The diff against the previous frame is what feeds the
/// scene-event log, so a crossing is recorded rather than silent.
function sceneMembers(w) {
  const b = sceneBox(w);
  const n = w.holon_atom_count();
  const inside = new Uint8Array(n);
  let count = 0;
  for (let i = 0; i < n; i++) {
    const ok = Math.abs(w.holon_atom_x(i) - b.cx) <= b.hx
      && Math.abs(w.holon_atom_y(i) - b.cy) <= b.hy
      && Math.abs(w.holon_atom_z(i) - b.cz) <= b.hz;
    inside[i] = ok ? 1 : 0;
    if (ok) count += 1;
  }
  const prev = State.sceneMembership;
  if (prev && prev.length === n) {
    for (let i = 0; i < n; i++) {
      if (prev[i] !== inside[i]) {
        State.sceneLog.push({
          frame: w.holon_frame(),
          atom: i,
          event: inside[i] ? "admitted" : "released",
        });
      }
    }
    // Bounded: the log is a live readout, not an audit trail that must survive the page.
    if (State.sceneLog.length > 400) State.sceneLog.splice(0, State.sceneLog.length - 400);
  }
  State.sceneMembership = inside;
  return { inside, count, total: n, box: b };
}

// ---------------------------------------------------------------- rendering
//
// The camera, orbit, dock and drawer are the mock's vocabulary and are kept deliberately:
// WB-7.2 records that the interaction prototype was the valuable half. What changed is
// underneath — every coordinate below is read from the engine each frame.

const canvas = $("webgl-canvas");
const ctx = canvas.getContext("2d");
let dpr = 1;

function resize() {
  dpr = Math.min(window.devicePixelRatio || 1, 2);
  canvas.width = Math.floor(window.innerWidth * dpr);
  canvas.height = Math.floor(window.innerHeight * dpr);
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
}

/// Project a point given in BOX-CENTRED bohr.
function project(x, y, z, vw, vh) {
  const { yaw, pitch, distance, fov } = State.camera;
  // WB-2.4c: the BOX is what tilts. Every point arrives in box coordinates and is rotated
  // by −θ before projection, so on screen the container leans and the world stays level —
  // which is what the physics says, because the engine holds an axis-aligned box and the
  // tilt lives entirely in the field's direction within it. Applying this here rather than
  // to the atoms means the box, its contents and the hand all lean together by
  // construction; there is no second place for the rotation to disagree with itself.
  if (State.tiltDeg !== 0) {
    const t = (-State.tiltDeg * Math.PI) / 180;
    const ct = Math.cos(t), st = Math.sin(t);
    const rx = x * ct - y * st;
    const ry = x * st + y * ct;
    x = rx;
    y = ry;
  }
  const x1 = x * Math.cos(yaw) - z * Math.sin(yaw);
  const z1 = x * Math.sin(yaw) + z * Math.cos(yaw);
  const y2 = y * Math.cos(pitch) - z1 * Math.sin(pitch);
  const z2 = y * Math.sin(pitch) + z1 * Math.cos(pitch);
  const zEye = z2 + distance;
  if (zEye <= 0.05) return null;
  const f = (vh * 0.8) / Math.tan((fov * Math.PI) / 360);
  return { sx: (x1 / zEye) * f + vw * 0.5, sy: -(y2 / zEye) * f + vh * 0.5, zEye, scale: f / zEye };
}

/// The scene's own extent in bohr, and the scale factor that maps it into camera units.
function sceneFrame() {
  const w = State.w;
  const width = w.holon_width();
  const height = w.holon_height();
  const depth = w.holon_depth();
  // Normalise the longest box edge to one camera unit, so the camera distance means the
  // same thing whatever box the engine is carrying.
  const span = Math.max(width, height, depth);
  return { width, height, depth, span, k: 1 / span };
}

function styleFor(z) {
  const sp = PALETTE.get(z);
  const h = PALETTE.get(1);
  if (!sp) return { colour: "#7fd1c0", radiusBohr: 0.69 };
  return { colour: sp.colour, radiusBohr: sp.radius_bohr };
}

function render3D() {
  const w = State.w;
  const vw = window.innerWidth;
  const vh = window.innerHeight;
  ctx.clearRect(0, 0, vw, vh);

  if (!State.booted) return;
  drawTemperatureGlow(vw, vh);
  const f = sceneFrame();
  // ZOOMING OUT past 1× shrinks the world box on screen by the same ratio, so the cube
  // band draws this scene as what it is at a kilometre — a speck — rather than filling the
  // viewport with twelve atoms wearing a kilometre's label. The scene box (the quotient)
  // never grows past the world box; only the drawing scale falls.
  f.k *= Math.min(1, State.zoom);
  const cx = 0.5 * f.width, cy = 0.5 * f.height, cz = 0.5 * f.depth;

  // THE SCENE-BOX CUT. Membership is computed once per frame and the draw list is the
  // subset — the world box keeps every holon and keeps simulating it, so this removes
  // nothing from the physics and everything it removes is recorded in the scene log.
  const mem = sceneMembers(w);
  State.sceneCount = mem.count;

  drawBox(f, cx, cy, cz, vw, vh);
  drawSceneBox(mem.box, f, cx, cy, cz, vw, vh);

  // Bonds first, from the engine's own pair readings. The BOND CRITERION is the engine's
  // (`E_rel < 0` and inside the outer turning point); this file draws the verdict and
  // does not own a distance threshold of its own.
  const np = w.holon_pair_count();
  for (let k = 0; k < np; k++) {
    if (w.holon_pair_bonded(k) !== 1) continue;
    const i = w.holon_pair_i(k), j = w.holon_pair_j(k);
    // A bond is drawn only when BOTH ends are in the scene. Drawing a bond to an atom
    // that is not on screen would draw a line to nowhere and imply a partner the view
    // does not contain.
    if (!mem.inside[i] || !mem.inside[j]) continue;
    const a = project((w.holon_atom_x(i) - cx) * f.k, (w.holon_atom_y(i) - cy) * f.k, (w.holon_atom_z(i) - cz) * f.k, vw, vh);
    const b = project((w.holon_atom_x(j) - cx) * f.k, (w.holon_atom_y(j) - cy) * f.k, (w.holon_atom_z(j) - cz) * f.k, vw, vh);
    if (!a || !b) continue;
    ctx.strokeStyle = "rgba(150, 225, 210, 0.55)";
    ctx.lineWidth = Math.max(1, 3 * Math.min(a.scale, b.scale) * 0.01);
    ctx.beginPath();
    ctx.moveTo(a.sx, a.sy);
    ctx.lineTo(b.sx, b.sy);
    ctx.stroke();
  }

  // Atoms, painter-sorted back to front.
  const n = w.holon_atom_count();
  const drawn = [];
  for (let i = 0; i < n; i++) {
    if (!mem.inside[i]) continue;
    const p = project((w.holon_atom_x(i) - cx) * f.k, (w.holon_atom_y(i) - cy) * f.k, (w.holon_atom_z(i) - cz) * f.k, vw, vh);
    if (!p) continue;
    drawn.push({ i, p, z: w.holon_atom_species_z(i) });
  }
  drawn.sort((a, b) => b.p.zEye - a.p.zEye);
  for (const d of drawn) {
    const st = styleFor(d.z);
    const r = Math.max(2, st.radiusBohr * f.k * d.p.scale);
    const grad = ctx.createRadialGradient(
      d.p.sx - r * 0.3, d.p.sy - r * 0.3, r * 0.1, d.p.sx, d.p.sy, r,
    );
    grad.addColorStop(0, "#ffffff");
    grad.addColorStop(0.35, st.colour);
    grad.addColorStop(1, "rgba(0,0,0,0.55)");
    ctx.fillStyle = grad;
    ctx.beginPath();
    ctx.arc(d.p.sx, d.p.sy, r, 0, Math.PI * 2);
    ctx.fill();
    if (d.i === State.hand.grabbed) {
      ctx.strokeStyle = "#ffd479";
      ctx.lineWidth = 2;
      ctx.stroke();
    }
  }

  // The hand's anchor and its tether, when the engine says something is held.
  if (w.holon_grabbed() >= 0) {
    const a = project((w.holon_anchor_x() - cx) * f.k, (w.holon_anchor_y() - cy) * f.k, (w.holon_anchor_z() - cz) * f.k, vw, vh);
    const g = w.holon_grabbed();
    const b = project((w.holon_atom_x(g) - cx) * f.k, (w.holon_atom_y(g) - cy) * f.k, (w.holon_atom_z(g) - cz) * f.k, vw, vh);
    if (a && b) {
      ctx.strokeStyle = "rgba(255, 212, 121, 0.8)";
      ctx.setLineDash([4, 4]);
      ctx.lineWidth = 1.5;
      ctx.beginPath();
      ctx.moveTo(a.sx, a.sy);
      ctx.lineTo(b.sx, b.sy);
      ctx.stroke();
      ctx.setLineDash([]);
    }
  }
}

/// WB-9.3 — the temperature glow. A READOUT and never a control: it is keyed to the
/// engine's MEASURED kinetic temperature, not to the thermostat's setpoint, so a scene
/// that is still settling glows at where it IS rather than where it was asked to go. The
/// thermostat panel remains the only control.
///
/// The scale is anchored at both ends by physical values rather than by taste: 273.15 K
/// (ice point) is fully blue and 373.15 K (boiling at 1 atm) is fully red, so the colour
/// says something about water rather than about a designer's gradient. Outside that range
/// it saturates, which is honest — the glow is an indicator, not a thermometer, and the
/// scene panel carries the number.
function drawTemperatureGlow(vw, vh) {
  const t = State.w.holon_temperature();
  if (!Number.isFinite(t)) return;
  const u = Math.max(0, Math.min(1, (t - 273.15) / (373.15 - 273.15)));
  const r = Math.round(40 + 180 * u);
  const b = Math.round(220 - 170 * u);
  const g = Math.round(90 + 40 * (1 - Math.abs(u - 0.5) * 2));
  const grad = ctx.createRadialGradient(vw / 2, vh / 2, 0, vw / 2, vh / 2, Math.max(vw, vh) * 0.7);
  grad.addColorStop(0, `rgba(${r}, ${g}, ${b}, 0.16)`);
  grad.addColorStop(1, "rgba(0, 0, 0, 0)");
  ctx.fillStyle = grad;
  ctx.fillRect(0, 0, vw, vh);
}

/// The SCENE box, drawn as its own frame so the cut is visible. A viewer who sees fewer
/// atoms than the panels report should be able to see WHY, and where the boundary is.
/// Omitted at zoom 1, where the scene box IS the world box and a second identical frame
/// would just be a heavier line.
function drawSceneBox(b, f, cx, cy, cz, vw, vh) {
  if (State.zoom <= 1.0001) return;
  const corners = [];
  for (const sx of [-1, 1]) for (const sy of [-1, 1]) for (const sz of [-1, 1]) {
    corners.push(project(sx * b.hx * f.k, sy * b.hy * f.k, sz * b.hz * f.k, vw, vh));
  }
  const edges = [[0,1],[0,2],[0,4],[1,3],[1,5],[2,3],[2,6],[3,7],[4,5],[4,6],[5,7],[6,7]];
  ctx.strokeStyle = "rgba(179, 136, 255, 0.55)";
  ctx.setLineDash([3, 3]);
  ctx.lineWidth = 1;
  for (const [a, bb] of edges) {
    if (!corners[a] || !corners[bb]) continue;
    ctx.beginPath();
    ctx.moveTo(corners[a].sx, corners[a].sy);
    ctx.lineTo(corners[bb].sx, corners[bb].sy);
    ctx.stroke();
  }
  ctx.setLineDash([]);
}

function drawBox(f, cx, cy, cz, vw, vh) {
  const inset = State.w.holon_wall_inset();
  const hx = (0.5 * f.width - inset) * f.k;
  const hy = (0.5 * f.height - inset) * f.k;
  const hz = (0.5 * f.depth - inset) * f.k;
  const corners = [];
  for (const sx of [-1, 1]) for (const sy of [-1, 1]) for (const sz of [-1, 1]) {
    corners.push(project(sx * hx, sy * hy, sz * hz, vw, vh));
  }
  const edges = [[0,1],[0,2],[0,4],[1,3],[1,5],[2,3],[2,6],[3,7],[4,5],[4,6],[5,7],[6,7]];
  ctx.strokeStyle = "rgba(120, 200, 190, 0.18)";
  ctx.lineWidth = 1;
  for (const [a, b] of edges) {
    if (!corners[a] || !corners[b]) continue;
    ctx.beginPath();
    ctx.moveTo(corners[a].sx, corners[a].sy);
    ctx.lineTo(corners[b].sx, corners[b].sy);
    ctx.stroke();
  }
}

// ---------------------------------------------------------------- telemetry

/// Panels that do not change frame to frame: the chart's provenance, the preset's served
/// set, the fences. Re-rendered on a scene change rather than every frame.
function renderStatics() {
  const w = State.w;
  const s = State.served;

  put("hud-mix-name", s.label);
  renderMixChips(s);

  // --- the chart in the viewport (WB-5.1)
  put("chart-pair-knots", String(w.holon_table_knots()));
  put("chart-re", `${w.holon_table_r_e().toFixed(6)} a₀`);
  put("chart-de", `${w.holon_table_d_e().toFixed(6)} Ha`);
  put("chart-referee",
    `${fmtSci(w.holon_chem_referee_residual())} Ha over ${w.holon_chem_referee_points()} separations`);
  tag("tag-chart", "live", "holon_table_knots / holon_table_r_e / holon_chem_referee_residual");

  // --- the pair bank, with the PRICE each curve actually cost.
  //
  // The price column is the point of this table rather than a decoration. Two rows can
  // both read SERVED and differ by ninety times in what they spent, and until this page
  // measured it nothing in the tree recorded that the same H-H curve is available at two
  // prices through two doors.
  const rows = s.pairs.map((p) => {
    const state = p.ok ? "SERVED" : p.door === "priced" ? "UNPAID" : "REFUSED";
    const route = p.route === 1 ? "determinant/FCI" : p.route === 2 ? "MPS/DMRG" : "none";
    const price = p.ms === undefined
      ? (p.door === "priced" ? "not spent" : "—")
      : p.ms < 1000 ? `${p.ms.toFixed(0)} ms` : `${(p.ms / 1000).toFixed(1)} s`;
    return `<tr class="${p.ok ? "ok" : "fenced"}">`
      + `<td>${sym(p.za)}–${sym(p.zb)}</td>`
      + `<td>${route}</td>`
      + `<td>${p.nDet.toExponential(2)}</td>`
      + `<td>${price}</td>`
      + `<td>${state}</td></tr>`;
  }).join("");
  if (UI["bank-rows"]) UI["bank-rows"].innerHTML = rows;

  // --- curves the engine permits but that have not been paid for
  if (UI["priced-list"]) {
    UI["priced-list"].innerHTML = s.priced.length
      ? s.priced.map((p) =>
        `<button class="btn-micro pay" data-pay="${p.key}">SOLVE ${sym(p.za)}–${sym(p.zb)}`
        + ` · ${p.nDet.toExponential(2)} determinants</button>`).join("")
      : "";
    for (const b of UI["priced-list"].querySelectorAll("[data-pay]")) {
      b.addEventListener("click", () => { loadPreset(State.mixture, { pay: b.dataset.pay }); });
    }
  }

  // --- the three-body sector
  put("trimer-state", s.trimer.state.toUpperCase());
  put("trimer-detail", s.trimer.detail);
  tag("tag-trimer", s.trimer.state === "refused" ? "fenced" : "live",
    s.trimer.state === "refused"
      ? "the H₃ generator declined this grid"
      : "holon_trimer_nodes / holon_trimer_peak / holon_fence_untabulated");

  // --- the fence register (WB-5.2)
  if (UI["fence-list"]) {
    UI["fence-list"].innerHTML = s.fences.length
      ? s.fences.map((f) => `<li><b>${f.what}</b><span>${f.why}</span>`
          + `<span class="fence-meta"><b>owner</b> ${f.owner || "UNREGISTERED"}`
          + ` · <b>exit</b> ${f.exit || "none stated"}`
          + ` · <b>register</b> ${f.register || "UNREGISTERED"}</span></li>`).join("")
      : `<li class="none"><b>none</b><span>every interaction this scene can produce is served by a certified chart.</span></li>`;
  }

  // --- device class & artifact (WB-5.4, M-DEVICE-CLASS)
  // The scale ladder (FSD-W3 §11.2). STRUCTURE only — the band names, what runs, the
  // fences and their citations do not change while a scene is loaded. The acuity figure
  // DOES change, with every camera move, so it is written per frame by `renderTelemetry`
  // into the slot left here. It lived in this function once and read the same 4,383 at
  // every zoom, which is not an acuity readout at all: a number that cannot respond to its
  // own input is a caption. Caught by varying the camera in the browser, not by reading.
  //
  // The FINE bands' state is not structure and is not written here either: it is a property
  // of the artifact (`bandLiveness`), so it is written per frame beside the acuity figure.
  // Writing it here would freeze it at the moment a preset loaded, and a band that went
  // live on the next reload rather than on the export landing is a band whose state is
  // about this page's history instead of about the engine.
  if (UI["ladder-rows"]) {
    const st = ladderStatus(w);
    UI["ladder-rows"].innerHTML = LADDER.map((b, i) => {
      const status = b.state === "live"
        ? `<span class="lad-live">CERTIFIED · LIVE</span>`
          + `<span class="lad-detail"><b>certificate</b> ${b.certifiedBy}</span>`
          + `<span class="lad-detail">${st.text}</span>`
        : b.state === "export-gated"
          // The state word itself is written per frame into `lad-state-${i}` below, because
          // it depends on the artifact. What is structural is the owner and the exit, and
          // they are shown whichever way the flip falls: a band that has just gone live
          // still owes the reader the debt it was carrying an hour ago.
          ? `<span class="lad-fenced" id="lad-state-${i}">FENCED — PENDING</span>`
            + `<span class="lad-detail"><b>owner</b> ${b.owner} · <b>exit</b> ${b.exit}</span>`
            + `<span class="lad-detail" id="lad-liveWhen-${i}">—</span>`
          : `<span class="lad-fenced">FENCED</span>`
            + `<span class="lad-detail"><b>owner</b> ${b.owner} · <b>exit</b> ${b.exit}</span>`
            + (b.positive
              ? `<span class="lad-detail lad-positive"><b>measured anyway</b> ${b.positive}</span>`
              : "");
      return `<div class="lad ${b.state}" id="lad-row-${i}"><div class="lad-head">`
        + `<b>${b.band}</b><span>${b.scale}</span></div>`
        + `<div class="lad-runs">${b.runs}</div>`
        + `<div class="lad-status">${status}</div>`
        + (b.readout
          ? `<div class="lad-status"><span class="lad-detail lad-readout">`
            + `<b>readout</b> ${b.readout}</span></div>`
          : "")
        + `<div class="lad-status"><span class="lad-acuity" id="lad-acuity-${i}">—</span></div>`
        + `<code>${b.cite}${b.certificate ? " · " + b.certificate : ""}`
        + `${b.measuredBy ? " · " + b.measuredBy : ""}`
        + `${b.positiveCite ? " · " + b.positiveCite : ""}`
        + `${b.readoutCite ? " · " + b.readoutCite : ""}`
        + `${b.declaredCite ? " · " + b.declaredCite : ""}`
        + `${b.buildCite ? " · " + b.buildCite : ""}`
        + `${b.ganttCite ? " · " + b.ganttCite : ""}</code></div>`;
    }).join("");
    // The slots were just created, so re-bind before anything writes to them.
    bindUI();
  }

  // The water story (WB-9.6), rendered from RECORD above so the citation and the digits
  // cannot drift apart in the markup.
  if (UI["record-rows"]) {
    UI["record-rows"].innerHTML = Object.values(RECORD).map((r) =>
      `<div class="rec"><div class="rec-head"><b>${r.value}</b><span>${r.what}</span></div>`
      + `<p>${r.note}</p><code>${r.cite}</code></div>`).join("");
  }

  // The not-served list, from data so the gate can read it (R3 full).
  if (UI["not-served-rows"]) {
    UI["not-served-rows"].innerHTML = NOT_SERVED.map((f) =>
      `<li><b>${f.what}</b><span>${f.why}</span>`
      + `<span class="fence-meta"><b>owner</b> ${f.owner} · <b>exit</b> ${f.exit}`
      + ` · <b>register</b> ${f.register}</span></li>`).join("");
  }

  put("manifest-device-class", State.deviceClass);
  put("manifest-sha", State.artifact.sha256);
  put("manifest-bytes", `${State.artifact.bytes.toLocaleString()} bytes`);
  put("manifest-substeps", `${fmtSci(w.holon_substeps_per_second(), 3)} substeps/s (measured on this device at load)`);
  put("manifest-capacity", `${w.holon_atom_count()} atoms in scene · engine capacity for this device ${Math.floor(w.holon_n_max())}`);
}

function renderTelemetry() {
  const w = State.w;
  if (!State.booted) return;

  // --- HUD ------------------------------------------------------------------
  put("hud-rate-val", fmtRate(State.rate.fsPerSec));
  put("hud-realtime", `${fmtSci(State.rate.pctRealtime, 2)} % realtime`);
  put("hud-fps-val", State.rate.fps ? State.rate.fps.toFixed(0) : "—");
  put("hud-scale-val", fmtLength(sceneFrame().span));
  put("hud-device-class", State.deviceClass);

  // --- the ledger (WB-4.3): every column, and the gate's verdict ------------
  put("led-kin", fmtEnergy(w.holon_e_kin()));
  put("led-pair", fmtEnergy(w.holon_e_pair()));
  put("led-three", fmtEnergy(w.holon_e_three()));
  put("led-wall", fmtEnergy(w.holon_e_wall()));
  put("led-spring", fmtEnergy(w.holon_e_spring()));
  put("led-grav", fmtEnergy(w.holon_e_grav()));
  put("led-wext", fmtEnergy(w.holon_w_ext()));
  put("led-total", fmtEnergy(w.holon_energy()));
  put("led-invariant", fmtEnergy(w.holon_ledger() - w.holon_ledger_origin()));
  put("led-drift", `${fmtSci(w.holon_drift())} Ha`);
  put("led-drift-peak", `${fmtSci(w.holon_drift_peak())} Ha`);
  put("led-bound", `${fmtSci(w.holon_drift_bound())} Ha`);
  const eGate = w.holon_energy_gate() === 1;
  put("led-gate", eGate ? "CLOSED" : "OPEN");
  if (UI["led-gate"]) UI["led-gate"].className = eGate ? "val green" : "val red";
  tag("tag-ledger", "live", "holon_e_kin / holon_e_pair / holon_w_ext / holon_drift / holon_energy_gate");

  put("mom-residual", fmtSci(w.holon_momentum_residual()));
  put("mom-bound", fmtSci(w.holon_momentum_bound()));
  const pGate = w.holon_momentum_gate() === 1;
  put("mom-gate", pGate ? "CLOSED" : "OPEN");
  if (UI["mom-gate"]) UI["mom-gate"].className = eGate ? "val green" : "val red";

  // --- the closure-defect lens (WB-5.3) ------------------------------------
  const rows = w.holon_row_count();
  let worst = 0, worstAt = 0;
  for (let k = 0; k < rows; k++) {
    worst = Math.max(worst, Math.abs(w.holon_row_closure_defect(k)));
    worstAt = Math.max(worstAt, Math.abs(w.holon_row_closure_defect_at_formation(k)));
  }
  put("clo-rows", String(rows));
  put("clo-worst", rows ? `${fmtSci(worst)} Ha` : "— (no holon in the scene yet)");
  put("clo-at-formation", rows ? `${fmtSci(worstAt)} Ha` : "—");
  put("clo-molecules", String(w.holon_census_molecules()));
  put("clo-formations", String(w.holon_census_formations()));
  put("clo-dissolutions", String(w.holon_census_dissolutions()));
  put("clo-rejections", String(w.holon_census_closure_rejections()));

  // The CERTIFIED THINGS, one row each (WB-9.5). This is the ontology on screen: each row
  // is a composite the census ADMITTED, with the closure defect it was admitted on beside
  // the one it carries now. A candidate whose defect exceeded its budget is not here — it
  // is in the rejection count above, which is why both are shown together.
  if (UI["census-rows"]) {
    const KINDS = ["pair", "triple", "cluster"];
    let html = "";
    for (let k = 0; k < Math.min(rows, 24); k++) {
      const kind = KINDS[w.holon_row_kind(k)] || `kind ${w.holon_row_kind(k)}`;
      html += `<tr><td>${k}</td><td>${kind}</td><td>${w.holon_row_member_count(k)}</td>`
        + `<td>${fmtSci(w.holon_row_e_bond(k), 2)}</td>`
        + `<td>${fmtSci(w.holon_row_closure_defect(k), 2)}</td>`
        + `<td>${fmtSci(w.holon_row_closure_defect_at_formation(k), 2)}</td></tr>`;
    }
    if (!rows) html = `<tr class="none"><td colspan="6">no composite has formed yet — the census admits nothing it has not measured</td></tr>`;
    else if (rows > 24) html += `<tr class="none"><td colspan="6">…and ${rows - 24} more</td></tr>`;
    UI["census-rows"].innerHTML = html;
  }
  tag("tag-closure", "live", "holon_row_closure_defect / holon_census_molecules");

  // --- the scene ------------------------------------------------------------
  put("scene-atoms", String(w.holon_atom_count()));
  put("scene-bonds", String(w.holon_bonded_count()));
  put("scene-clusters", `${w.holon_cluster_count()} over ${w.holon_cluster_atoms()} atoms`);
  put("scene-temp", tempIn(w.holon_temperature()));
  put("scene-time", `${(w.holon_time() * AU_TO_FS).toFixed(3)} fs`);
  put("scene-steps", w.holon_steps().toLocaleString());
  tag("tag-scene", "live", "holon_atom_count / holon_bonded_count / holon_temperature");

  // --- the clocks (WB-1.4 / WB-2.3) ----------------------------------------
  put("clk-dt", `${w.holon_dt().toFixed(4)} a.u. (${(w.holon_dt() * AU_TO_FS).toFixed(5)} fs)`);
  put("clk-omega-dt", w.holon_omega_dt().toFixed(4));
  put("clk-requested", fmtRate(w.holon_sim_speed()));
  put("clk-delivered", fmtRate(State.rate.fsPerSec));
  put("clk-dilation", `${(100 * w.holon_dilation()).toFixed(1)} %`);
  const rung = ["EXACT", "TIME-DILATED", "ACCURACY DECLARED", "REFUSED"][w.holon_rung()] || "—";
  put("clk-rung", rung);
  tag("tag-clocks", "live", "holon_dt / holon_sim_speed / holon_dilation / holon_rung");

  // --- the fence counter (WB-5.2) ------------------------------------------
  //
  // PER FORCE PASS, not cumulative: the engine zeroes it at the top of every pass and
  // re-counts. So it is a property of the SCENE rather than of how long you have watched,
  // and it is exactly the combinatorial count of untabulated triples the composition
  // admits — 55 for one oxygen among eleven hydrogens, which is C(11,2); 164 for four
  // oxygens among eight hydrogens, which is 4*C(8,2) + C(4,2)*8 + C(4,3). A counter that
  // climbed with wall time would be a different and much less useful quantity.
  put("fence-count", w.holon_fence_untabulated().toLocaleString());
  tag("tag-fence", "live", "holon_fence_untabulated");

  // --- the thermostat pill (WB-3.3) ----------------------------------------
  //
  // A scene that cannot step is NOT settling, and saying so was the specific defect the
  // first browser run caught: a frozen O:2H box read "SETTLING · 0.0 K → 293.1 K", which
  // describes a process that is not happening in words that suggest it soon will.
  const t = w.holon_temperature();
  const running = State.served && State.served.stepsAllowed;
  const settling = running && State.thermostatOn && Math.abs(t - State.targetK) > 0.05 * State.targetK;
  put("settling-label", !running
    ? "NOT STEPPING · a curve this scene needs is fenced"
    : settling
      ? `SETTLING · ${tempIn(t)} → ${tempIn(State.targetK)}`
      : State.thermostatOn ? `THERMOSTATTED · ${tempIn(t)}` : `FREE (NVE) · ${tempIn(t)}`);
  if (UI["settling-pill"]) {
    UI["settling-pill"].className = !running ? "settling-pill halted"
      : settling ? "settling-pill settling" : "settling-pill settled";
  }

  // --- gravity (WB-2.4), the tier-separation exhibit -----------------------
  const gAu = w.holon_gravity();
  const gx = w.holon_gravity_x(), gy = w.holon_gravity_y(), gz = w.holon_gravity_z();
  const available = w.holon_gravity_available() === 1;
  put("grav-field", available
    ? `${State.gravityG.toFixed(2)} G  (${fmtSci(gAu, 3)} a₀/aut²)`
    : "REFUSED — a wrapping box has no bottom");
  put("grav-vector", `(${fmtSci(gx, 2)}, ${fmtSci(gy, 2)}, ${fmtSci(gz, 2)})`);
  put("grav-tilt", `${State.tiltDeg.toFixed(0)}° — the box, not the world`);
  put("grav-energy", fmtEnergy(w.holon_e_grav()));
  // The exhibit itself, computed from the engine's own constant and its own k_B, at the
  // scene's CURRENT box height rather than a fixed 1 nm — so the number moves when the
  // scale does, which is the whole point of calling it a tier-separation exhibit.
  const boxBohr = sceneFrame().height;
  const kt = K_B_HA * Math.max(w.holon_temperature(), 1e-9);
  const uDrop = M_H_ME * gAu * boxBohr;
  put("grav-vs-kt", kt > 0 ? `${fmtSci(uDrop / kt, 2)} × kT over the box` : "—");
  tag("tag-grav", available ? "live" : "fenced",
    available
      ? "holon_gravity_x/y/z / holon_e_grav / holon_g_earth"
      : "this boundary wraps, and a linear potential is not well-posed on a torus");

  // --- pressure, on the landed door (WB-2.2) -------------------------------
  const pDefined = w.holon_pressure_defined() === 1;
  put("press-value", pDefined ? `${fmtSci(w.holon_pressure(), 4)} Ha/a₀³` : "NOT A PRESSURE");
  put("press-box", `${w.holon_width().toFixed(1)} × ${w.holon_height().toFixed(1)} × ${w.holon_depth().toFixed(1)} a₀  (${State.boxScale.toFixed(3)}× reference)`);
  put("press-hand", fmtEnergy(w.holon_w_ext()));
  put("press-note", pDefined
    ? "The virial pressure of the scene. A READOUT, never a setpoint: the control is the box, and this is what the box is doing."
    : "Under walls the virial is contaminated by the wall term, so the engine declines to call this number a pressure. The box control still works; the readout does not.");
  tag("tag-press", pDefined ? "live" : "fenced",
    pDefined ? "holon_pressure / holon_width / holon_w_ext"
             : "holon_pressure_defined reports the virial is not a pressure under these walls");
  if (State.lastScaleRefusal) put("press-refusal", State.lastScaleRefusal);

  // --- the two boxes, stated separately because they are never the same knob ----
  const sb = sceneBox(w);
  put("scene-zoom", `${State.zoom.toFixed(2)}×`);
  put("scene-centre", State.camera.target
    ? `${sb.cx.toFixed(1)}, ${sb.cy.toFixed(1)}, ${sb.cz.toFixed(1)} a₀ — aimed (double-click to move)`
    : `${sb.cx.toFixed(1)}, ${sb.cy.toFixed(1)}, ${sb.cz.toFixed(1)} a₀ — world centre (double-click to aim)`);
  put("world-extent", `${w.holon_width().toFixed(1)} × ${w.holon_height().toFixed(1)} × ${w.holon_depth().toFixed(1)} a₀`);
  put("scene-extent", `${(2 * sb.hx).toFixed(1)} × ${(2 * sb.hy).toFixed(1)} × ${(2 * sb.hz).toFixed(1)} a₀ (world ÷ zoom)`);
  const drawn = State.sceneCount ?? w.holon_atom_count();
  const total = w.holon_atom_count();
  put("scene-drawn", `${drawn} of ${total}`);
  // AN EMPTY SCENE BOX MUST EXPLAIN ITSELF. Zooming into a sparse world finds vacuum, and
  // that is a true fact about the world rather than a broken view — but a blank screen
  // that says nothing is indistinguishable from a bug, and WB-7 does not stop applying
  // because the failure is aesthetic. Measured here: this opener places holons on a
  // 6-bohr shell, so the box CENTRE is empty by construction and a scene box smaller than
  // the shell contains nothing at all.
  const meanSep = total > 1 ? Math.cbrt((w.holon_width() * w.holon_height() * w.holon_depth()) / total) : 0;
  put("scene-outside", drawn === total
    ? "nothing is outside the scene box"
    : drawn === 0
      ? `ALL ${total} are outside — the scene box is ${(2 * sb.hx).toFixed(1)} a₀ across and this `
        + `world averages one holon per ${meanSep.toFixed(1)} a₀, so at this zoom you are `
        + `looking at vacuum. The physics is unchanged; zoom out or move the view centre.`
      : `${total - drawn} outside the scene box — still simulated, still in every whole-only number below`);
  // The last few crossings, so a removal is a recorded event rather than a vanishing.
  const recent = State.sceneLog.slice(-4).reverse()
    .map((e) => `frame ${e.frame}: atom ${e.atom} ${e.event}`).join(" · ");
  put("scene-events", recent || "no crossing yet");
  tag("tag-boxes", "live", "holon_width / holon_atom_x / the scene-box membership diff");

  // --- the scale ladder's live half (§9c's acuity law) ----------------------
  const viewM = viewSpanMetres(w);
  const descending = descentActive(w);
  const pick = pinnedAtomIndex(w);
  put("ladder-view", `view span ${fmtMetres(viewM)} · acuity is the allocator`
    + (descending ? " · past the molecular band — one atom pinned" : ""));
  // THE LADDER'S OWN TAG, and it is the weakest band's. This element existed and was never
  // written, so it read "—" at every zoom — a panel-level honesty tag that says nothing is
  // the shape WB-7.1 is about, one altitude up from the digits.
  const liveBands = LADDER.filter((b) => b.state === "live"
    || (b.state === "export-gated" && bandLiveness(b.liveWhen, hasExport).live)).length;
  tag("tag-ladder", liveBands === LADDER.length ? "live" : "fenced",
    liveBands === LADDER.length
      ? "every band on the ladder runs its certified chart"
      : `${liveBands} of ${LADDER.length} bands run; the rest carry their debt, owner and `
        + "exit, and each names the build paying it");
  LADDER.forEach((b, i) => {
    // THE FINE BANDS' STATE, written from the artifact every frame rather than from the
    // source. `bandLiveness` is the whole rule and it runs here: name every export the band
    // is missing, or say it is live because none are.
    if (b.state === "export-gated") {
      const lv = bandLiveness(b.liveWhen, hasExport);
      const stateEl = UI[`lad-state-${i}`];
      if (stateEl) {
        stateEl.textContent = lv.live ? "LIVE — every export it needs resolves" : "FENCED — PENDING";
        stateEl.className = lv.live ? "lad-live" : "lad-fenced";
      }
      const whenEl = UI[`lad-liveWhen-${i}`];
      if (whenEl) {
        whenEl.innerHTML = lv.live
          ? `<b>serves</b> ${b.liveWhen.join(", ")} — all present in this artifact`
          : `<b>debt</b> ${lv.missing.length} of ${b.liveWhen.length} exports are not in `
            + `this artifact yet: ${lv.missing.join(", ")}. No digit for them is drawn.`;
      }
      // The row's own colour follows the flip too. A band reading LIVE inside a rose fence
      // is a panel disagreeing with itself, and the eye sorts a drawer by colour before it
      // reads a word.
      const rowEl = UI[`lad-row-${i}`];
      if (rowEl) rowEl.className = `lad ${lv.live ? "live" : "export-gated"}`;
    }
    const pop = acuityPopulation(viewM, b.lengthM);
    const el = UI[`lad-acuity-${i}`];
    if (!el) return;
    // A PINNED BAND REPORTS THE PIN, NOT THE COUNT. Acuity's cubic figure is the number of
    // holons this view could distinguish at that scale; on the fine bands that figure runs
    // to 10¹² and reporting it beside a band the page seeds with exactly ONE would invite
    // the reading the acuity law exists to refuse. The law's own rule for these bands is
    // the seed: pin one holon of that tier near the view centre.
    if (b.pinned) {
      el.className = pick.index >= 0 ? "lad-acuity" : "lad-acuity zero";
      el.textContent = pick.index < 0
        ? "no atom in the scene to pin"
        : `ONE pinned — atom ${pick.index} (${sym(w.holon_atom_species_z(pick.index))}), `
          + `${pick.how}; acuity would admit ${pop.toLocaleString()} at this view and the `
          + "page allocates one"
          + (descending ? " · the view is past the molecular band"
            : " · the view has not yet zoomed past the molecular band, and a hand on an "
              + "atom would arrive here too");
      return;
    }
    // THE FLOOR ALLOCATES NOTHING, and reporting an acuity figure for it would say the
    // opposite. Below the nucleus this page has no tier to populate — that is the fence,
    // and a number beside it would read as a population being carried.
    if (b.noAllocation) {
      el.className = "lad-acuity zero";
      el.textContent = "below the nucleus — this page allocates nothing here, and the "
        + "fence above says who is building what would go here";
      return;
    }
    el.className = pop === 0 ? "lad-acuity zero" : "lad-acuity";
    if (pop === 0) {
      el.textContent = "below this band's scale — nothing to allocate";
      return;
    }
    let line = `acuity admits ${pop.toLocaleString()}${pop === 1 ? " — the pinned seed" : ""}`;
    // On the LIVE band, say what the engine is actually carrying beside what acuity
    // admits. Without it the row reads "acuity admits 2.6e20" next to "CERTIFIED · LIVE"
    // and invites exactly the reading the whole design refuses — that the page is
    // allocating them. The GAP between the two numbers is the de-allocation law's job,
    // so it belongs on screen rather than in the paragraph underneath.
    if (b.state === "live") {
      const carried = w.holon_atom_count();
      line += ` · engine carries ${carried.toLocaleString()}`;
      if (pop > carried) line += " — the rest is out of view and stays coarse";
    }
    el.textContent = line;
  });

  renderDescent(w, descending, pick);
  renderTierRail(w, viewM);
  put("gov-delivered",
    `delivered ${fmtRate(State.rate.fsPerSec)} · ${fmtSci(State.rate.pctRealtime, 2)} % realtime`);
}

// ------------------------------------- the ladder's readouts (FSD-W3 §11.2, WB-10.3)
//
// The bottom of the ladder, and the one page-side arithmetic §11.2 grants at the top of it.
// Every row here is one entry of `DESCENT_FIELDS`, and every row's digits — or its refusal
// to draw digits — follow that entry's declared source. Three rules are absolute:
//
//   * a row whose export is not in this artifact shows NO NUMBER. It shows the export's
//     name. That is the whole of WB-7 at this altitude: there is no third option between a
//     traced number and a fence, and a placeholder is not a fence.
//   * a DECLARED input is labelled DECLARED beside its digits, never LIVE — including when
//     it arrives through an export. `holon_nucleus_mass_u` returning a measured mass does
//     not make the mass computed (WB-1.7).
//   * "in a molecule or free" comes from the census export and from nowhere else. The
//     page has every atom's coordinates and could measure a separation; §11.2's WB-1.6
//     forbids exactly that, because a distance in JavaScript is not the census's bond
//     criterion and a page that guessed would be asserting the engine's own verdict.
function renderDescent(w, descending, pick) {
  const i = pick.index;
  const z = i >= 0 ? w.holon_atom_species_z(i) : 0;
  const species = PALETTE.get(z);

  // THE CARD'S OWN TAG is the WEAKEST row's, not the best — a card that said LIVE while
  // six of its rows were waiting on an export would be advertising the half that works.
  const pending = PENDING_EXPORTS.filter((e) => !hasExport(e.name));
  tag("tag-descent", pending.length === 0 && State.lawProbe ? "live" : "fenced",
    pending.length === 0 && State.lawProbe
      ? "every export these rows need resolves in this artifact"
      : `${pending.length} export(s) WB-10.1/WB-10.2 is building are not in this artifact `
        + `yet (${pending.map((e) => e.name).join(", ") || "none"})`
        + (State.lawProbe ? "" : ", and law_probe.json is not in the tree")
        + ". Those rows name what they wait for and draw no digits; the rest are live or "
        + "declared and say which.");

  // --- the pick (WB-1.6) ---------------------------------------------------
  descField("desc-pinned",
    i < 0 ? "pending" : "live",
    i < 0
      ? "no atom in the scene to pin"
      : `atom ${i} — ${sym(z)} · ${pick.how}`
        + (descending ? " · the view is at the fine bands"
          : " · the view has not yet zoomed past the molecular band"),
    "holon_grabbed / holon_atom_x,y,z / holon_atom_species_z");

  // ONE HELPER, and every export-served row below goes through it. `paint` writes what
  // `exportRow` decided; nothing here decides for itself whether it has an export, because
  // nine hand-written guards is nine chances for a missing `!` to draw a number the engine
  // never returned.
  const paint = (id, row) => descField(id, row.kind, row.text, row.trace);
  const owed = (spec, why) =>
    `${spec} is in build (lead, engine). ${why} Until the rebuilt wasm carries it this row `
    + "shows the export's name and no digits.";

  paint("desc-membership", exportRow("holon_atom_in_molecule", hasExport, "live",
    () => {
      if (i < 0) return "—";
      const r = w.holon_atom_in_molecule(i);
      return r === 0 ? "FREE — in no census molecule row" : `IN A MOLECULE — census row ${r - 1}`;
    },
    "holon_atom_in_molecule — the census's own membership, never a distance in this file",
    owed("WB-10.1", "WB-1.6 asks for the census's verdict; a separation measured in "
      + "JavaScript is not the census's bond criterion and the page will not substitute one.")));

  // --- the atom band (WB-10.2) ---------------------------------------------
  //
  // THE SOLVE IS RUN, then read back. The four getters below return the LAST solve for that
  // atom and return zeros with exit 4 for an atom that has not been solved — so the page
  // runs the solve first (throttled, never per frame) and then honours the exit code.
  maybeSolveAtomBand(w, i);
  const EXITS = ["converged", "iteration cap", "stagnated", "trivial", "not computed"];
  const bandExit = hasExport("holon_atom_band_exit") && i >= 0 ? w.holon_atom_band_exit(i) : 4;
  // WB-5.2: NEVER SILENTLY ZEROED. `holon_atom_band_energy` returns exactly 0.0 for an atom
  // whose solve was never kept, and 0.0 hartree is a number a reader would take for an
  // energy. When the engine's own exit says "not computed" the three value rows say so
  // instead of printing its zeros — the refusal is the engine's and the page displays it.
  const solved = bandExit !== 4;
  const bandRows = [
    ["desc-band-energy", "holon_atom_band_energy", (v) => fmtEnergy(v)],
    ["desc-band-electrons", "holon_atom_band_n_electrons",
      (v) => `${v} electron${v === 1 ? "" : "s"}`],
    ["desc-band-residual", "holon_atom_band_residual", (v) => `${fmtSci(v, 3)} Ha`],
  ];
  for (const [id, name, fmt] of bandRows) {
    paint(id, exportRow(name, hasExport, "live",
      () => (i < 0 ? "—"
        : !solved ? "NOT COMPUTED for this atom — the engine's own exit code 4, not a zero"
          : `${fmt(w[name](i))}  · at frame ${State.atomBand.atFrame.toLocaleString()}`
            + ` (${State.atomBand.atTimeFs.toFixed(3)} fs)`),
      `${name} — read back from holon_atom_band_solve, on the lane engine in this page`,
      owed("WB-10.2", "the picked atom's STO-3G FCI on the lane engine in wasm, gated "
        + "bit-identical against the native referee by `tests/wasm_law.rs`.")));
  }
  paint("desc-band-exit", exportRow("holon_atom_band_exit", hasExport, "live",
    () => (i < 0 ? "—" : `${EXITS[bandExit] || `code ${bandExit}`} (${bandExit})`),
    "holon_atom_band_exit — how the last solve for this atom ended",
    owed("WB-10.2", "the solve's own exit code; without it the page cannot tell a converged "
      + "energy from a slot that was never filled.")));

  // --- the nucleus band (WB-10.1, WB-1.7) ----------------------------------
  //
  // DECLARED, all of it except the wavelength. These are measured inputs the Hamiltonian
  // never computes, so the tag says DECLARED whether the digits come from the committed
  // species table or from the engine reading its own copy of that table back to us.
  descField("desc-nuc-z", i < 0 ? "pending" : "live",
    i < 0 ? "—" : `Z = ${z}`, "holon_atom_species_z");

  // THE TWO ABSENCES ARE DIFFERENT FACTS and the row says which. "The table did not load"
  // is this page's own failure; "no isotope is declared for this element" is a true
  // statement about the element. One message for both told the reader the wrong thing in
  // whichever case it was not written for.
  descField("desc-nuc-isotope",
    species && species.isotope ? "declared" : "pending",
    species && species.isotope ? species.isotope
      : PALETTE.size === 0
        ? "FENCED — species_palette.json did not load, so this page has no isotope table"
        : `FENCED — no isotope is declared for Z = ${z} in the committed species table`,
    "species_palette.json#isotope — a measured input, not a computed one (WB-1.7)");

  // MASS: two doors to one measured input, and the TAG DOES NOT MOVE between them. The
  // committed species table carries it and so does the engine; either way it is DECLARED —
  // a function returning a measured mass does not make the mass computed, and calling it
  // LIVE because an export served it is exactly the costume WB-1.7 names.
  //
  // THE THREE DECLARED ROWS FENCE INDEPENDENTLY, because the engine's two tables do not
  // cover the same elements. Measured against the shipped artifact: at Z = 11 the mass door
  // serves 22.989769282 u while the spin and charge-radius doors both return their
  // sentinels. A single "the nucleus is declared" flag would have fenced a mass the engine
  // was serving, or shown a spin it was not.
  const massU = hasExport("holon_nucleus_mass_u") && i >= 0
    ? declaredPositive(w.holon_nucleus_mass_u(z))
    : species && species.mass_u > 0 ? species.mass_u : null;
  descField("desc-nuc-mass", massU === null ? "pending" : "declared",
    massU === null
      ? "FENCED — no mass is declared for this element (the door returned its sentinel)"
      : `${massU.toFixed(9)} u`,
    hasExport("holon_nucleus_mass_u")
      ? "holon_nucleus_mass_u — a DECLARED measured input read back through the engine"
      : "species_palette.json#mass_u — a DECLARED measured input (WB-1.7)");

  paint("desc-nuc-spin", exportRow("holon_nucleus_spin2", hasExport, "declared",
    () => {
      if (i < 0) return "—";
      const s2 = declaredU32(w.holon_nucleus_spin2(z));
      if (s2 === null) return null;
      return s2 % 2 === 0 ? `I = ${s2 / 2}` : `I = ${s2}/2`;
    },
    "holon_nucleus_spin2 — twice the spin, a DECLARED measured input (WB-1.7)",
    owed("WB-10.1", "`holon_chem::elements::NUCLEI` declares the spin with its source and the door "
      + "ships it.")));

  paint("desc-nuc-radius", exportRow("holon_nucleus_charge_radius_fm", hasExport, "declared",
    () => {
      if (i < 0) return "—";
      const fm = declaredPositive(w.holon_nucleus_charge_radius_fm(z));
      return fm === null ? null : `${fm.toFixed(4)} fm`;
    },
    "holon_nucleus_charge_radius_fm — a DECLARED measured input (WB-1.7)",
    owed("WB-10.1", "`holon_chem::elements::NUCLEI` declares the charge radius with its source. A charge radius this "
      + "page invented would be the WB-7 lie in a new costume.")));

  paint("desc-nuc-lambda", exportRow("holon_nucleus_thermal_wavelength_bohr", hasExport, "live",
    () => {
      if (i < 0) return "—";
      const lam = w.holon_nucleus_thermal_wavelength_bohr(i);
      // The engine returns 0 where the scene's temperature is undefined, and zero is a
      // LENGTH here — printing it would assert a point-like nucleus. The undefined case
      // says so instead.
      return lam > 0
        ? `${fmtLength(lam)}  (${lam.toFixed(6)} a₀, at ${tempIn(w.holon_temperature())})`
        : "UNDEFINED — the scene has no temperature to evaluate it at";
    },
    "holon_nucleus_thermal_wavelength_bohr — COMPUTED by the engine in closed form at "
      + "the scene's own holon_temperature",
    owed("WB-10.1", "gated by `tests/nucleus.rs`. The page holds the mass, k_B and the "
      + "temperature and does NOT evaluate the closed form itself: two implementations of "
      + "one number are how they start disagreeing.")));

  // --- the bit-identity gate (WB-10.2) -------------------------------------
  const wasmBits = hasExport("holon_law_probe") ? f64Bits(w.holon_law_probe()) : null;
  paint("desc-probe-wasm", exportRow("holon_law_probe", hasExport, "live",
    () => `0x${wasmBits}  (${w.holon_law_probe().toPrecision(17)})`,
    "holon_law_probe — the reference solve, run in this artifact",
    owed("WB-10.2", "a fixed reference solve whose bits are pinned natively. Without it "
      + "the page has nothing to compare, and a comparison with nothing is not a gate.")));

  const native = State.lawProbe;
  descField("desc-probe-native", native ? "declared" : "pending",
    native ? `0x${native.energy_bits_hex.toLowerCase()}  (pinned by ${native.pinned_by})`
      : "FENCED — PENDING law_probe.json",
    native ? `law_probe.json#energy_bits_hex — ${native.probe}`
      : "the engine lane writes `law_probe.json` beside this page out of `tests/wasm_law.rs`. "
        + `It is not in the tree yet (${State.lawProbeWhy || "absent"}), so this row pends.`);

  const verdict = lawProbeVerdict(wasmBits, native && native.energy_bits_hex);
  descField("desc-probe-verdict",
    verdict === null ? "pending" : verdict ? "computed" : "computed",
    verdict === null
      ? "FENCED — PENDING both halves of the comparison"
      : verdict ? "EQUAL TO THE BIT" : "MISMATCH — the wasm and the native referee disagree",
    "a string comparison of two 64-bit patterns; there is no tolerance in this row");
  if (UI["desc-probe-verdict"]) {
    UI["desc-probe-verdict"].classList.toggle("red", verdict === false);
    UI["desc-probe-verdict"].classList.toggle("green", verdict === true);
  }

  // --- the cube band's hydrostatic column (§11.2, WB-2.4a) -----------------
  //
  // ARITHMETIC ON MEASURED CONSTANTS, and the label says so on screen. It is not a claim
  // about water: this scene's density is this scene's, and it is nothing like a kilometre
  // of liquid water. WB-2.4a's own figure for that — ~9.8 MPa at 1 km, about 97 atmospheres
  // — is the exhibit's point and is a MEASURED figure from `tests/gravity.rs`, not from
  // here. The two are shown as what they are: the same arithmetic on two different
  // substances, which is exactly the tier-separation exhibit.
  const rho = sceneDensitySI(w);
  const gSI = gravitySI(w);
  descField("desc-density", rho === null ? "pending" : "computed",
    rho === null
      ? "FENCED — the species table did not load, so there are no masses to sum"
      : `${rho.toFixed(1)} kg/m³ — ${w.holon_atom_count()} atoms in the world box`,
    "Σ mᵢ from species_palette.json#mass_me over holon_width × holon_height × holon_depth");

  const hViewM = 2 * sceneBox(w).hy * BOHR_TO_M;
  const hydro = (h) => (rho === null || gSI === null) ? null : rho * gSI * h;
  descField("desc-hydro-view",
    hydro(hViewM) === null ? "pending" : "computed",
    hydro(hViewM) === null
      ? (gSI === null
        ? "FENCED — the field is refused on a wrapping box, and a column with no down has no ρ g h"
        : "FENCED — no density to multiply")
      : `${fmtPressure(hydro(hViewM))}  over ${fmtMetres(hViewM)} of this scene`,
    "ρ (above) × g (holon_g_earth × the slider) × the scene box's own vertical extent");

  descField("desc-hydro-km",
    hydro(1.0e3) === null ? "pending" : "computed",
    hydro(1.0e3) === null
      ? (gSI === null
        ? "FENCED — the field is refused on a wrapping box"
        : "FENCED — no density to multiply")
      : `${fmtPressure(hydro(1.0e3))}  over the cube band's 1 km, at THIS scene's density`,
    "the same arithmetic at the cube band's own declared depth. Liquid water gives ~9.8 MPa "
    + "there (WB-2.4a, measured in tests/gravity.rs); this scene is not liquid water.");
}

// ---------------------------------------------------------------- the hand (WB-4)
//
// The hand is a SPRING TERM in the engine's Hamiltonian with a time-dependent anchor, and
// moving the anchor posts exactly dU into `w_ext`. That is why the energy gate stays
// closed through a drag: the hand's work is a receipt column, not an excuse (WB-4.3).

function pointerToWorld(px, py) {
  // Pick on the plane through the box centre that faces the camera. The anchor is a 3D
  // point, so a 2D pointer has to choose a depth; choosing the centre plane keeps the
  // grab in the middle of the box where the atoms are.
  const w = State.w;
  const f = sceneFrame();
  const vw = window.innerWidth, vh = window.innerHeight;
  const { yaw, pitch, distance, fov } = State.camera;
  const fscale = (vh * 0.8) / Math.tan((fov * Math.PI) / 360);
  const zEye = distance;
  const x1 = ((px - vw * 0.5) * zEye) / fscale;
  const y2 = (-(py - vh * 0.5) * zEye) / fscale;
  // Invert the pitch then the yaw, with z1 = 0 on the chosen plane.
  const z2 = 0;
  const y = y2 * Math.cos(pitch) + z2 * Math.sin(pitch);
  const z1 = -y2 * Math.sin(pitch) + z2 * Math.cos(pitch);
  const x = x1 * Math.cos(yaw) + z1 * Math.sin(yaw);
  const z = -x1 * Math.sin(yaw) + z1 * Math.cos(yaw);
  return {
    x: x / f.k + 0.5 * f.width,
    y: y / f.k + 0.5 * f.height,
    z: z / f.k + 0.5 * f.depth,
  };
}

/// WB-4.1: the grab radius is a fixed fraction of the viewport edge, in physical units,
/// and is DISPLAYED. At this tier that is a few atoms.
function grabRadiusBohr() {
  return 0.05 * sceneFrame().span;
}

function tryGrab(px, py) {
  const w = State.w;
  const p = pointerToWorld(px, py);
  const r = grabRadiusBohr();
  State.hand.radiusBohr = r;
  // `holon_nearest_atom` searches in the scene's x/y plane; the anchor is then placed in
  // full 3D, which is what `holon_move_anchor_3d` is for.
  const i = w.holon_nearest_atom(p.x, p.y, r);
  if (i >= 0) {
    w.holon_grab(i);
    State.hand.grabbed = i;
    if (UI["reticle-hud"]) UI["reticle-hud"].classList.remove("hidden");
    put("reticle-text", `grab ${fmtLength(r)} · atom ${i} (${sym(w.holon_atom_species_z(i))})`);
  }
  return i >= 0;
}

function dragTo(px, py) {
  if (State.hand.grabbed < 0) return;
  const p = pointerToWorld(px, py);
  State.w.holon_move_anchor_3d(p.x, p.y, p.z);
}

function releaseHand() {
  if (State.hand.grabbed < 0) return;
  State.w.holon_release();
  State.hand.grabbed = -1;
  if (UI["reticle-hud"]) UI["reticle-hud"].classList.add("hidden");
}

// ---------------------------------------------------------------- determinism (WB-5.4)

/// Run the seeded scene a fixed number of substeps from a fresh reset and digest the
/// state. Two runs on the same device class must agree bit for bit; the page shows both
/// digests and the verdict rather than asserting the property.
///
/// The digest is over the RAW f64 bits of every coordinate and velocity-derived speed, so
/// a difference of one ulp changes it. A digest over rounded decimals would agree across
/// runs that are not in fact identical, which is the vacuous-success shape.
function replayDigest() {
  const w = State.w;
  const preset = PRESETS[State.mixture];
  w.holon_reset(State.atomsRequested);
  applyComposition(preset, w.holon_atom_count());
  w.holon_reset(State.atomsRequested);
  applyComposition(preset, w.holon_atom_count());
  w.holon_rebase();
  for (let f = 0; f < 200; f++) w.holon_step_frame(64);

  const n = w.holon_atom_count();
  const buf = new Float64Array(n * 4);
  for (let i = 0; i < n; i++) {
    buf[i * 4 + 0] = w.holon_atom_x(i);
    buf[i * 4 + 1] = w.holon_atom_y(i);
    buf[i * 4 + 2] = w.holon_atom_z(i);
    buf[i * 4 + 3] = w.holon_atom_speed(i);
  }
  // FNV-1a over the raw bytes. A non-cryptographic digest is the right tool: this is a
  // bit-identity check between two runs in the same page, not a claim anybody has to
  // trust against an adversary.
  const bytes = new Uint8Array(buf.buffer);
  let h = 0x811c9dc5;
  for (let i = 0; i < bytes.length; i++) {
    h ^= bytes[i];
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return { digest: h.toString(16).padStart(8, "0"), steps: w.holon_steps(), energy: w.holon_energy() };
}

function runReplayCheck() {
  const a = replayDigest();
  const b = replayDigest();
  State.replay = { last: b, prev: a, matched: a.digest === b.digest };
  put("replay-a", a.digest);
  put("replay-b", b.digest);
  put("replay-verdict", State.replay.matched
    ? `BIT-IDENTICAL on ${State.deviceClass}`
    : "DIVERGED — the seeded scene is not reproducible on this device class");
  if (UI["replay-verdict"]) UI["replay-verdict"].className = State.replay.matched ? "val green" : "val red";
  put("replay-detail", `${a.steps.toLocaleString()} substeps, E = ${fmtEnergy(a.energy)}`);
  // The check left the scene at the end of its second run; put the user's scene back.
  loadPreset(State.mixture);
}

// ---------------------------------------------------------------- input wiring

let orbiting = false;
let lastX = 0, lastY = 0;
let pointerDownAt = 0;

function initInput() {
  canvas.addEventListener("mousedown", (e) => {
    pointerDownAt = performance.now();
    lastX = e.clientX;
    lastY = e.clientY;
    // Shift (or the secondary button) orbits; a plain press reaches for an atom, and
    // falls through to orbit when it finds none.
    if (e.shiftKey || e.button === 2 || !tryGrab(e.clientX, e.clientY)) orbiting = true;
  });
  window.addEventListener("mousemove", (e) => {
    if (State.hand.grabbed >= 0) {
      dragTo(e.clientX, e.clientY);
    } else if (orbiting) {
      State.camera.yaw += (e.clientX - lastX) * 0.006;
      State.camera.pitch = Math.max(-1.4, Math.min(1.4, State.camera.pitch + (e.clientY - lastY) * 0.006));
    }
    lastX = e.clientX;
    lastY = e.clientY;
  });
  window.addEventListener("mouseup", () => {
    releaseHand();
    orbiting = false;
  });
  // AIM. Double-click puts the view centre on whatever you clicked — the observer's axis,
  // never the hand's: this calls nothing on the engine and moves no atom.
  canvas.addEventListener("dblclick", (e) => {
    const p = pointerToWorld(e.clientX, e.clientY);
    State.camera.target = { x: p.x, y: p.y, z: p.z };
  });
  canvas.addEventListener("contextmenu", (e) => e.preventDefault());
  canvas.addEventListener("wheel", (e) => {
    e.preventDefault();
    State.camera.distance = Math.max(0.6, Math.min(12, State.camera.distance * (1 + e.deltaY * 0.001)));
  }, { passive: false });

  // Touch: one finger orbits or grabs, two pinch the camera.
  let pinch0 = 0;
  canvas.addEventListener("touchstart", (e) => {
    if (e.touches.length === 1) {
      const t = e.touches[0];
      lastX = t.clientX; lastY = t.clientY;
      if (!tryGrab(t.clientX, t.clientY)) orbiting = true;
    } else if (e.touches.length === 2) {
      pinch0 = Math.hypot(e.touches[0].clientX - e.touches[1].clientX, e.touches[0].clientY - e.touches[1].clientY);
    }
  }, { passive: true });
  canvas.addEventListener("touchmove", (e) => {
    if (e.touches.length === 1) {
      const t = e.touches[0];
      if (State.hand.grabbed >= 0) dragTo(t.clientX, t.clientY);
      else if (orbiting) {
        State.camera.yaw += (t.clientX - lastX) * 0.008;
        State.camera.pitch = Math.max(-1.4, Math.min(1.4, State.camera.pitch + (t.clientY - lastY) * 0.008));
      }
      lastX = t.clientX; lastY = t.clientY;
    } else if (e.touches.length === 2 && pinch0 > 0) {
      const d = Math.hypot(e.touches[0].clientX - e.touches[1].clientX, e.touches[0].clientY - e.touches[1].clientY);
      State.camera.distance = Math.max(0.6, Math.min(12, State.camera.distance * (pinch0 / d)));
      pinch0 = d;
    }
  }, { passive: true });
  canvas.addEventListener("touchend", () => { releaseHand(); orbiting = false; pinch0 = 0; }, { passive: true });

  window.addEventListener("resize", resize);
  resize();
}

// ------------------------------------------------ the surface (FSD-W3 §11.5)
//
// A cube, four selectors, the tier as the zoom, everything else under ☰. What follows is
// the rail's rendering; the controls' DOORS are unchanged and live in `initHUD` below.

/// THE TIER RAIL: zoom is the tier selector. Each band on LADDER is a stop, placed each
/// frame at the zoom that puts the view span at the band's scale (the world box moves
/// under the hand, so the stops move with it); the active tier is the band nearest the
/// view span in log distance; the card carries the band's state from the same rule the
/// drawer's ladder uses (`bandLiveness`), so the two can never disagree.
function renderTierRail(w, viewM) {
  const zoomEl = UI["sheet-zoom"];
  if (!zoomEl) return;
  const min = Number(zoomEl.min);
  const max = Number(zoomEl.max);
  const spanM = Math.max(w.holon_width(), w.holon_height(), w.holon_depth()) * BOHR_TO_M;
  if (!State.tierStopsBuilt && UI["tier-stops"]) {
    UI["tier-stops"].innerHTML = LADDER.map((b, i) =>
      `<button class="tier-stop" id="tier-stop-${i}" title="${b.band} · ${b.scale}">`
      + `<b>${b.scale}</b><span>${b.band}</span></button>`).join("");
    LADDER.forEach((b, i) => {
      const el = document.getElementById(`tier-stop-${i}`);
      UI[`tier-stop-${i}`] = el;
      el.addEventListener("click", () => {
        // the stop sets the same ratio the slider sets, and nothing else
        const v = Math.min(max, Math.max(min, Math.log10(spanM / b.lengthM)));
        zoomEl.value = v.toFixed(2);
        State.zoom = Math.pow(10, v);
        put("sheet-zoom-val", `${State.zoom.toFixed(2)}×`);
      });
    });
    State.tierStopsBuilt = true;
  }
  // THE ACTIVE BAND IS THE COARSEST ONE THE VIEW CAN RESOLVE: the first band (LADDER runs
  // coarse to fine) whose scale fits inside the view span. This is §9c's acuity rule, the
  // same one `descentActive` applies at the molecular band — not "nearest in log
  // distance", which read a 2 nm view of twelve atoms as the H-bond network band, a
  // fenced band, while the scene was running the molecular band's live chart.
  let active = LADDER.length - 1;
  for (let i = 0; i < LADDER.length; i++) {
    if (LADDER[i].lengthM <= viewM) { active = i; break; }
  }
  // stops sit at their own zoom; two within a label's height of each other (the nucleus
  // and the fold below it are half a decade apart on a nineteen-decade axis) are pushed
  // apart top-down so both stay legible — the slider's thumb, not the label, is the truth
  const railPx = UI["tier-stops"] ? UI["tier-stops"].clientHeight : 0;
  const minGapPct = railPx > 0 ? (100 * 30) / railPx : 0;
  let lastPct = -Infinity;
  const pcts = LADDER.map((b) => {
    const v = Math.log10(spanM / b.lengthM);
    let pct = Math.min(99, Math.max(1, 100 * (v - min) / (max - min)));
    if (pct < lastPct + minGapPct) pct = lastPct + minGapPct;
    lastPct = pct;
    return pct;
  });
  // the same gap from the bottom up, so the last stops are not folded onto the rail's end
  for (let i = pcts.length - 1; i >= 0; i--) {
    pcts[i] = Math.min(pcts[i], 99 - (pcts.length - 1 - i) * minGapPct);
  }
  pcts.forEach((pct, i) => {
    const el = UI[`tier-stop-${i}`];
    if (el) el.style.top = `${pct.toFixed(1)}%`;
  });
  const hasExport = (name) => typeof w[name] === "function";
  LADDER.forEach((b, i) => {
    const el = UI[`tier-stop-${i}`];
    if (!el) return;
    const live = b.state === "live"
      || (b.state === "export-gated" && bandLiveness(b.liveWhen, hasExport).live);
    el.className = `tier-stop${live ? "" : " fenced"}${i === active ? " active" : ""}`;
  });
  if (active < 0) return;
  const b = LADDER[active];
  const lv = b.state === "export-gated" ? bandLiveness(b.liveWhen, hasExport) : null;
  const live = b.state === "live" || (lv && lv.live);
  put("band-name", b.band.toUpperCase());
  put("band-scale", `${b.scale} · view span ${fmtMetres(viewM)}`);
  tag("tag-band", live ? "live" : "fenced",
    live ? `this band runs its certified chart — ${b.runs}`
      : lv ? `FENCED — PENDING: ${lv.missing.join(", ")} not in this artifact`
        : `FENCED — owner ${b.owner}`);
  put("band-text", live
    ? b.runs
    : `${b.runs}. FENCED — ${b.owner}. ${b.readout ? `Served meanwhile: ${b.readout}.` : ""}`);
}

const DENSITY_MARKS = [
  { name: "air", gcc: 1.2e-3 },
  { name: "liquid water", gcc: 1.0 },
  { name: "ice VII", gcc: 1.65 },
  { name: "white dwarf", gcc: 1.0e6 },
  { name: "neutronium", gcc: 4.0e14 },
];

/// The size slider's readout and the density axis under it. The marks are placed on the
/// slider's own axis (density goes as the inverse cube of the edge), so "liquid water" is
/// wherever THIS scene's mass in THIS box would reach 1 g/cm³ — a computed position, not
/// a label at a fixed pixel. Marks past the slider's floor are listed off-axis with the
/// decades they are past it, which is how neutronium appears: a limit on the box size,
/// fourteen decades below the chart's electrons.
function renderSizeAxis() {
  const w = State.w;
  const rho = w ? sceneDensitySI(w) : null;
  if (rho == null) {
    put("sheet-size-val", "—");
    return;
  }
  const gcc = rho / 1000;
  put("sheet-size-val",
    `${fmtSci(gcc, 2)} g/cm³ · box ×${(1 / State.boxScale).toFixed(2)} · ${w.holon_width().toFixed(1)} a₀`);
  const el = UI["density-axis"];
  const slider = UI["sheet-size"];
  if (el && slider) {
    const min = Number(slider.min);
    const max = Number(slider.max);
    const vNow = -Math.log10(State.boxScale);
    const on = [];
    const off = [];
    for (const m of DENSITY_MARKS) {
      const v = vNow + Math.log10(m.gcc / gcc) / 3;
      if (v >= min && v <= max) {
        const pct = 100 * (v - min) / (max - min);
        // alternate rows so neighbours (liquid water and ice VII are a fifth of a decade
        // apart) do not print on top of each other
        on.push(`<span class="dmark" style="left:${pct.toFixed(1)}%;top:${(on.length % 2) * 9}px" `
          + `title="${m.name}: ${m.gcc} g/cm³">${m.name}</span>`);
      } else if (v > max) {
        off.push(`${m.name} +${(3 * (v - max)).toFixed(0)}`);
      }
    }
    el.innerHTML = on.join("")
      + (off.length
        ? `<span class="dmark off" title="decades of density past the slider's floor — and past `
          + `this chart's electrons; a limit on the box size, not a stop">beyond the floor: ${off.join(", ")} decades</span>`
        : "");
  }
  put("size-fence", State.lastScaleRefusal
    ? State.lastScaleRefusal
    : "floor: STO-3G electrons, no degeneracy pressure");
}

function scaleRefusalText(code) {
  return code === 91
    ? "REFUSED: that factor is not a positive finite number."
    : code === 92
      ? "REFUSED: that scale would collapse the box onto its own walls."
      : code === 93
        ? "REFUSED: that box could not hold its own periodic images."
        : `REFUSED by the engine, code ${code}.`;
}

/// Put the size slider where the box actually is.
function syncSizeSlider() {
  if (UI["sheet-size"]) UI["sheet-size"].value = (-Math.log10(State.boxScale)).toFixed(2);
}

/// The mixture chips' second lines: the active preset's served state, and for the others
/// whether their curves are in the tree — SHIPPED, in-browser, or absent by name.
function renderMixChips(s) {
  const w = State.w;
  for (const key of Object.keys(PRESETS)) {
    const p = PRESETS[key];
    let line;
    if (key === State.mixture) {
      line = s.stepsAllowed ? `running · ${State.atomsActual} atoms` : "cannot step";
    } else {
      const missing = p.pairs.filter(([za, zb]) =>
        !SHIPPED.pairs[`${za},${zb}`] && w && w.holon_bank_pair_is_heavy(za, zb) === 1);
      line = missing.length
        ? `needs ${missing.map(([za, zb]) => `${sym(za)}–${sym(zb)}`).join(", ")} shipped`
        : "ready";
    }
    put(`mix-lbl-${key}`, line);
  }
  put("mix-state", s.stepsAllowed ? `${s.label} · stepping` : `${s.label} · fenced`);
}

function initHUD() {
  // ☰ is everything that is not the cube, the four selectors or the tier rail.
  UI["btn-menu"]?.addEventListener("click", () => UI["telemetry-drawer"].classList.toggle("open"));
  UI["close-telemetry"]?.addEventListener("click", () => UI["telemetry-drawer"].classList.remove("open"));
  UI["btn-toggle-manifest"]?.addEventListener("click", () => UI["manifest-modal"].classList.toggle("hidden"));
  UI["btn-close-manifest"]?.addEventListener("click", () => UI["manifest-modal"].classList.add("hidden"));
  UI["btn-play-pause"]?.addEventListener("click", (e) => {
    State.paused = !State.paused;
    e.currentTarget.textContent = State.paused ? "▶" : "⏸";
    // A pause stops the clock window too, so the rate readout does not average a stall
    // into the delivered figure and report a slowdown that is not the engine's.
    State.clockWindow = { t0: null, simFs0: 0, frames: 0 };
  });

  UI["sheet-temp"]?.addEventListener("input", (e) => {
    State.targetK = Number(e.target.value);
    put("sheet-temp-val", tempIn(State.targetK));
    applyControls();
  });
  UI["sheet-thermostat"]?.addEventListener("change", (e) => {
    State.thermostatOn = e.target.checked;
    applyControls();
  });
  for (const pill of document.querySelectorAll(".u-pill")) {
    pill.addEventListener("click", () => {
      State.tempUnit = pill.dataset.unit;
      for (const p of document.querySelectorAll(".u-pill")) p.classList.toggle("active", p === pill);
      put("sheet-temp-val", tempIn(State.targetK));
    });
  }
  UI["sheet-grav"]?.addEventListener("input", (e) => {
    State.gravityG = Number(e.target.value);
    put("sheet-grav-val", `${State.gravityG.toFixed(2)} G`);
    applyControls();
    // A changed potential moves the total energy, so the ledger's origin moves with it —
    // otherwise the drift would be measured against an origin taken before this field
    // existed and would read a JUMP no integrator produced.
    State.w.holon_rebase();
  });
  UI["sheet-zoom"]?.addEventListener("input", (e) => {
    // ZOOM IS A RATIO. It changes the quotient and touches nothing in the engine — no
    // Sim call here at all, which is the two-box law's whole point and is what the
    // density-invariance gate checks.
    State.zoom = Math.pow(10, Number(e.target.value));
    put("sheet-zoom-val", `${State.zoom.toFixed(2)}×`);
  });
  UI["sheet-tilt"]?.addEventListener("input", (e) => {
    State.tiltDeg = Number(e.target.value);
    put("sheet-tilt-val", `${State.tiltDeg.toFixed(0)}°`);
    applyControls();
    // Changing the field's DIRECTION changes the potential, so the origin moves with it —
    // same reason changing its strength does.
    State.w.holon_rebase();
  });
  // WB-2.2: the control IS the box. Each press applies a multiplicative factor through the
  // engine's own door, which posts the move's cost to both ledger columns and refuses a
  // collapse by name. This file never touches the box itself.
  for (const btn of document.querySelectorAll("[data-boundary]")) {
    btn.addEventListener("click", () => {
      State.boundary = Number(btn.dataset.boundary);
      for (const b of document.querySelectorAll("[data-boundary]")) {
        b.classList.toggle("active", b === btn);
      }
      State.w.holon_set_boundary(State.boundary);
      // A changed boundary is a changed chart: the wall term appears or vanishes and the
      // field may be refused, so the ledger's origin has to move with it.
      applyControls();
      State.w.holon_rebase();
    });
  }
  for (const [id, factor] of [["btn-compress", 0.98], ["btn-expand", 1.02]]) {
    UI[id]?.addEventListener("click", () => {
      const code = State.w.holon_box_scale(factor);
      if (code === 1) {
        State.boxScale *= factor;
        State.lastScaleRefusal = null;
      } else {
        State.lastScaleRefusal = scaleRefusalText(code);
      }
      syncSizeSlider();
      renderSizeAxis();
    });
  }
  // THE SIZE SLIDER IS THE HAND ON THE WORLD BOX (WB-2.2), through the same door as the
  // buttons above: an absolute position on a log axis of compression, applied as the
  // multiplicative factor from where the box is now. A refusal snaps the slider back to
  // the box the engine actually has and says why — the slider never shows a size the
  // world does not have. The axis under it is DENSITY: the floor is where this chart's
  // electrons stop, and neutronium is a mark fourteen decades past it, not a stop.
  UI["sheet-size"]?.addEventListener("input", (e) => {
    const w = State.w;
    if (!w) return;
    const target = Math.pow(10, -Number(e.target.value));
    const factor = target / State.boxScale;
    const code = w.holon_box_scale(factor);
    if (code === 1) {
      State.boxScale = target;
      State.lastScaleRefusal = null;
    } else {
      State.lastScaleRefusal = scaleRefusalText(code);
      syncSizeSlider();
    }
    renderSizeAxis();
  });
  UI["sheet-gov"]?.addEventListener("input", (e) => {
    // Logarithmic: the useful range spans decades, and a linear slider would spend all
    // its travel at the fast end.
    State.govBias = Math.pow(10, Number(e.target.value));
    put("sheet-gov-val", `${State.govBias.toFixed(2)}× · requested ${fmtRate(State.baseSimSpeed * State.govBias)}`);
    applyControls();
  });
  UI["sheet-atoms"]?.addEventListener("input", (e) => {
    State.atomsRequested = Number(e.target.value);
    put("sheet-atoms-val", String(State.atomsRequested));
  });
  UI["sheet-atoms"]?.addEventListener("change", () => loadPreset(State.mixture));

  for (const chip of document.querySelectorAll(".mix-chip")) {
    chip.addEventListener("click", () => {
      for (const c of document.querySelectorAll(".mix-chip")) c.classList.toggle("active", c === chip);
      loadPreset(chip.dataset.mix);
    });
  }
  UI["btn-quick-reset"]?.addEventListener("click", () => loadPreset(State.mixture));
  UI["btn-replay"]?.addEventListener("click", runReplayCheck);
}

// ---------------------------------------------------------------- start

window.addEventListener("DOMContentLoaded", () => {
  bindUI();
  initInput();
  initHUD();
  boot().catch((err) => {
    State.bootError = String(err && err.message ? err.message : err);
    document.body.dataset.engine = "failed";
    put("boot-error", State.bootError);
    UI["boot-failure"]?.classList.remove("hidden");
  });
});
