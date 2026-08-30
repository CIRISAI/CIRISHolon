// Browser shell for the atom renderer.
//
// This file owns input and pixels. It owns NO physics: every force, energy, bond
// reading and gate verdict below is read out of the wasm module, which is the same
// rlib the native tests exercise. Where a number appears on screen it was computed in
// Rust, never recomputed here — a second implementation in JS would be a second thing
// to keep true.
//
// The potential is fetched and parsed HERE because the host already has a JSON parser
// and the wasm should not have to carry one. The knots are pushed through the same
// `holon_table_*` ABI the native loader uses.

const stage = document.querySelector("#stage");
const ctx = stage.getContext("2d");
const curveCanvas = document.querySelector("#curve");
const curveCtx = curveCanvas.getContext("2d");

const ui = Object.fromEntries(
  [
    "runtime-status", "stage-status", "clock", "provenance", "error-text",
    "energy-gate", "e-kin", "e-pair", "e-three", "e-wall", "e-spring", "w-ext", "ledger",
    "drift", "drift-bound", "drift-fill", "drift-ratio",
    "momentum-gate", "p-mag", "p-res", "p-bound",
    "pair-r", "pair-erel", "pair-router", "bond-count",
    "t-knots", "t-re", "t-asym", "t-res", "t-res-alt",
    "atom-count-out", "target-t-out", "temperature",
    "rung", "rung-note", "dt", "dt-ref", "omega-e", "omega-env", "omega-dt",
    "frame-ms", "substeps-frame", "sim-speed", "vib-wall", "dilation",
    "c-atoms", "c-molecules", "c-candidates", "c-global", "c-formations",
    "c-dissolutions", "c-rejections", "c-bondsector",
    "d-sps", "d-pps", "d-req", "d-nmax",
    "t3-nodes", "t3-peak", "t3-rmax", "t3-ms", "t3-note",
    "sim-speed-out", "dt-mult-out",
    "palette-provenance", "palette-rule", "spawn-note",
    "sp-name", "sp-z", "sp-mass", "sp-basis", "sp-pair", "sp-ndet", "sp-sep", "sp-radius",
    "brush-note", "bank-summary", "bank-rows", "bank-rule", "fence-note",
    "bank-refusal-note", "d1-note",
  ].map((id) => [id, document.querySelector(`#${id}`)]),
);

const controls = {
  reset: document.querySelector("#reset-button"),
  atomCount: document.querySelector("#atom-count"),
  simSpeed: document.querySelector("#sim-speed-control"),
  openBoundary: document.querySelector("#open-boundary"),
  thermostat: document.querySelector("#thermostat"),
  targetT: document.querySelector("#target-t"),
  allowDtGrowth: document.querySelector("#allow-dt-growth"),
  dtMult: document.querySelector("#dt-mult"),
  censusEnabled: document.querySelector("#census-enabled"),
};

const state = {
  w: null,
  world: { width: 40, height: 24 },
  radius: 0.6,
  dragging: false,
  // Clock 2 lives here: the wall interval is MEASURED between animation frames. Nothing
  // in this file assumes 60 Hz, or any Hz -- a display that runs at 120, or a tab the
  // browser is throttling, both just report a different dt and the accumulator copes.
  lastFrameMs: null,
  frameMs: 0,
  substepsThisFrame: 0,
  baseSimSpeed: 0,
  // The periodic palette, as `holon-chem` computed it. Null until the file loads; every
  // read below tolerates that, because a sandbox that cannot draw a palette must still
  // draw atoms.
  palette: null,
  byZ: new Map(),
  selectedZ: 1,
  // Does the simulation know what an element IS? Feature-detected on the wasm's own ABI
  // rather than assumed, so this file needs no edit on the day the sim lane lands it.
  speciesAware: false,
};

// ---------------------------------------------------------------- loading
//
// TWO routes to the same table, and the engine-computed one is the default.
//
// `holon_table_generate` asks the wasm to SOLVE the H2 potential -- STO-3G full CI from
// closed-form Gaussian integrals, analytic forces and curvature -- and push the knots
// straight into its own interpolator. Nothing is fetched and nothing is parsed. The file
// route below stays as a fallback for a host that cannot run the generator, or for an
// A/B against a different curve, and both routes end in the same interpolator and the
// same sign-convention check.

const GRID = { rMin: 0.3, rMax: 10.0, knots: 492 };

const LOAD_STATUS = [
  "empty", "ok", "too many knots", "too few knots", "R not increasing",
  "non-finite value", "the generator refused the requested grid",
];

async function loadPotential(w) {
  if (typeof w.holon_table_generate === "function") {
    const t0 = performance.now();
    const status = w.holon_table_generate(GRID.rMin, GRID.rMax, GRID.knots);
    const ms = performance.now() - t0;
    if (status === 1) {
      reportTable(w);
      const residual = w.holon_chem_referee_residual();
      const digest = (w.holon_chem_referee_digest() >>> 0).toString(16).padStart(8, "0");
      const points = w.holon_chem_referee_points();
      showProvenance(
        "ENGINE-COMPUTED (STO-3G FCI, f64)",
        `${GRID.knots} knots solved in ${ms.toFixed(0)} ms. Referee residual: ` +
        `max |dE| <= ${residual.toExponential(1)} Eh over ${points} separations, against ` +
        `the 50-digit curve 0x${digest}. R_e, D_e and the dissociation asymptote are ` +
        `computed here, not quoted. EXACT-IN-MODEL for STO-3G, not a prediction of ` +
        `experiment.`,
        false,
      );
      return { generated: true, ms };
    }
    // Fall through to the file. Saying so is the point: a silent fallback would leave
    // the viewer showing a curve nobody asked for and no way to notice.
    showError(
      `the engine generator was refused (${LOAD_STATUS[status] ?? status}); ` +
      `falling back to h2_potential.json.`,
    );
  }

  const response = await fetch("h2_potential.json");
  if (!response.ok) {
    throw new Error(
      `h2_potential.json: HTTP ${response.status}. Serve this directory over http (see README); ` +
      `a file:// page cannot fetch it.`,
    );
  }
  const file = await response.json();
  const R = file.R_grid_bohr, E = file.E_hartree, F = file.F_hartree_per_bohr;
  if (!Array.isArray(R) || !Array.isArray(E) || !Array.isArray(F)) {
    throw new Error("h2_potential.json is missing R_grid_bohr / E_hartree / F_hartree_per_bohr");
  }
  if (R.length !== E.length || R.length !== F.length) {
    throw new Error(`contract violation: array lengths ${R.length}/${E.length}/${F.length}`);
  }
  if (!w.holon_table_begin(R.length)) {
    throw new Error(`the table refused ${R.length} knots`);
  }
  const D2 = file.d2E_hartree_per_bohr2 ?? file.E2_hartree_per_bohr2;
  for (let i = 0; i < R.length; i += 1) {
    if (!w.holon_table_knot(i, R[i], E[i], F[i])) throw new Error(`the table refused knot ${i}`);
    if (Array.isArray(D2) && D2.length === R.length) w.holon_table_knot_curvature(i, D2[i]);
  }
  const status = w.holon_table_finish(file.R_e, file.D_e, file.E_asymptote);
  if (status !== 1) {
    throw new Error(`the table refused the curve: ${LOAD_STATUS[status] ?? status}`);
  }
  reportTable(w);

  const provenance = String(file.provenance ?? "");
  showProvenance(
    /placeholder/i.test(provenance) ? "PLACEHOLDER CURVE" : "FILE CURVE",
    provenance,
    /placeholder/i.test(provenance),
  );
  return { generated: false, file };
}

// The THREE-BODY surface, solved at load by the same engine.
//
// Feature-detected rather than assumed, so this file needs no edit on a build that does
// not carry it, and a refusal is SAID rather than swallowed: a sandbox running with the
// pairwise force loop alone is a different physics, and the panel should not imply
// otherwise. The cost is reported because it is the one load-time number this term adds
// and it is not small -- thousands of full-CI solves, not the pair curve's hundreds.
function loadTrimer(w) {
  if (typeof w.holon_trimer_generate !== "function") {
    ui["t3-nodes"].textContent = "absent";
    ui["t3-note"].textContent =
      "This build carries no three-body table: the force loop is pairwise-additive, so a "
      + "cluster has nothing to stop it growing. The ledger below is still closed; it is "
      + "closed around a different physics.";
    return;
  }
  const t0 = performance.now();
  const ok = w.holon_trimer_generate();
  const ms = performance.now() - t0;
  if (ok !== 1) {
    ui["t3-nodes"].textContent = "refused";
    showError("the three-body generator refused; the sandbox is running pairwise-additive.");
    return;
  }
  ui["t3-nodes"].textContent = w.holon_trimer_nodes().toLocaleString();
  ui["t3-peak"].textContent = `${w.holon_trimer_peak().toFixed(4)} Eh`;
  ui["t3-rmax"].textContent = `${w.holon_trimer_r_max().toFixed(1)} a0`;
  ui["t3-ms"].textContent = `${ms.toFixed(0)} ms`;
}

// Surface the sign-convention check rather than trusting it. `residual` assumes
// dE/dR = -F; `residual_alt` is the same statistic with the opposite hypothesis and sits
// near 2.0 for any consistent table. If they ever swap, the table means the other thing
// and the curve being simulated is mirrored. This runs on BOTH routes: the generator is
// no more exempt from it than a file is.
function reportTable(w) {
  const res = w.holon_table_residual();
  const alt = w.holon_table_residual_alt();
  ui["t-knots"].textContent = w.holon_table_knots();
  ui["t-re"].textContent = `${w.holon_table_r_e().toFixed(4)} a0 / ${w.holon_table_d_e().toFixed(5)} Eh`;
  ui["t-asym"].textContent = `${w.holon_table_asymptote().toFixed(6)} Eh`;
  ui["t-res"].textContent = res.toExponential(2);
  ui["t-res-alt"].textContent = alt.toExponential(2);
  if (!(alt > 20 * res)) {
    showError(
      `the supplied table cannot certify its sign convention (residual ${res.toExponential(2)} ` +
      `vs ${alt.toExponential(2)}); the curve may be mirrored.`,
    );
  }
}

function showProvenance(label, text, warn) {
  const banner = document.querySelector("#provenance-banner");
  banner.hidden = false;
  banner.classList.toggle("warn", Boolean(warn));
  document.querySelector("#provenance-label").textContent = label;
  ui.provenance.textContent = text;
}

function showError(message) {
  const banner = document.querySelector("#error-banner");
  banner.hidden = false;
  ui["error-text"].textContent = message;
}

// ---------------------------------------------------------------- the species palette
//
// `species_palette.json` is GENERATED by `holon-chem` (`examples/emit_palette.rs`), not
// authored here. Each entry carries a radius the engine derived from that element's own
// computed homonuclear curve and a colour from a declared ramp in Z, and it carries the
// two rules as strings so the panel can state which is which. Nothing in this file
// decides how big lithium is.
//
// The palette is a SELECTOR, and today it selects into a simulation that only knows about
// hydrogen. That is not hidden: `state.speciesAware` is feature-detected from the wasm's
// own exports, and when it is false the page says the selector cannot spawn yet and why.
// See ELEMENTS_SIM_API_REQUEST.md for the extension it is waiting on.

const BOHR = 0.529177210903; // angstrom per bohr, for the readout only

async function loadPalette() {
  try {
    const response = await fetch("species_palette.json");
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const data = await response.json();
    state.palette = data;
    for (const sp of data.species) state.byZ.set(sp.Z, sp);
    buildPalette(data);
    ui["palette-provenance"].textContent = data.provenance;
  } catch (error) {
    // A missing palette is not a broken sandbox: hydrogen still works, and saying so is
    // better than an empty strip with no explanation.
    ui["palette-provenance"].textContent = `palette unavailable (${error.message}) — hydrogen only`;
  }
}

function buildPalette(data) {
  const host = document.querySelector("#palette");
  host.textContent = "";
  // Discs are scaled so the LARGEST element fills the box; the ratios between them are
  // the computed ones, which is what makes the strip a reading rather than an icon set.
  const rMax = Math.max(...data.species.map((s) => s.radius_bohr));
  for (const sp of data.species) {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "swatch";
    button.setAttribute("role", "radio");
    button.setAttribute("aria-checked", String(sp.Z === state.selectedZ));
    button.dataset.z = sp.Z;
    button.dataset.bound = String(sp.homonuclear_bound);
    button.title = `${sp.symbol} — radius ${sp.radius_bohr.toFixed(3)} a0, ${sp.radius_rule}`;

    const box = document.createElement("span");
    box.className = "disc-box";
    const disc = document.createElement("span");
    disc.className = "disc";
    const px = Math.max(9, Math.round(42 * (sp.radius_bohr / rMax)));
    disc.style.width = `${px}px`;
    disc.style.height = `${px}px`;
    disc.style.background = sp.colour;
    box.append(disc);

    const label = document.createElement("span");
    label.className = "label";
    label.innerHTML = `<b>${sp.symbol}</b> <i>${sp.Z}</i>`;

    button.append(box, label);
    button.addEventListener("click", () => selectSpecies(sp.Z));
    host.append(button);
  }
  selectSpecies(state.selectedZ);
}

function selectSpecies(z) {
  state.selectedZ = z;
  for (const button of document.querySelectorAll(".swatch")) {
    button.setAttribute("aria-checked", String(Number(button.dataset.z) === z));
  }
  const sp = state.byZ.get(z);
  if (!sp) return;
  ui["sp-name"].textContent = `${sp.symbol} (${sp.isotope})`;
  ui["sp-z"].textContent = sp.Z;
  ui["sp-mass"].textContent = `${sp.mass_me.toFixed(1)} mₑ`;
  ui["sp-basis"].textContent = sp.n_basis;
  ui["sp-pair"].textContent = sp.homonuclear_bound
    ? `${sp.symbol}₂ binds, D_e = ${sp.homonuclear_D_e.toExponential(3)} Ha`
    : `${sp.symbol}₂ does NOT bind`;
  ui["sp-ndet"].textContent = sp.homonuclear_n_determinants.toLocaleString();
  ui["sp-sep"].textContent = `${sp.homonuclear_separation_bohr.toFixed(4)} a₀`;
  ui["sp-radius"].textContent =
    `${sp.radius_bohr.toFixed(4)} a₀ (${(sp.radius_bohr * BOHR).toFixed(3)} Å)`;
  if (ui["brush-note"]) {
    ui["brush-note"].textContent = state.speciesAware
      ? `SHIFT-click (or ALT-click) an atom on the stage to make it ${sp.symbol}. `
        + "A plain click still grabs and drags. The bank below then holds one curve per "
        + "species pair the scene forms, and the scene will not step until every one of "
        + "them is loaded."
      : "This build cannot put a chosen element in the scene; see the banner above.";
  }
  ui["palette-rule"].textContent =
    `radius — ${sp.radius_rule}. colour — ${sp.colour_rule}. ` +
    (sp.homonuclear_bound
      ? "The separation above is the root of dE/dR on the engine's own curve."
      : `No minimum deeper than ${state.palette.well_min_depth_hartree} Ha exists on the ` +
        `curve, so the size falls back to where the repulsion reaches ` +
        `${state.palette.contact_energy_hartree} Ha above the asymptote. That is the ` +
        `in-model truth, not a special case: nothing in the engine knows which elements ` +
        `are noble.`);
}

// ---------------------------------------------------------------- the pair-table bank
//
// The engine holds one curve per unordered species pair. This section does three things:
// puts a chosen element into the scene, makes sure every pair the scene now forms has a
// curve, and DISPLAYS what each curve says about itself.
//
// The split is the engine's, not this page's: `holon_bank_pair_is_heavy` answers whether
// a pair may be solved at load, and heavy pairs are fetched from `tables/`. Asking the
// engine rather than re-deriving the rule here is what keeps the page and the gate from
// disagreeing about which pairs are affordable.

/// Shipped curves, by "ZaZb" with Za <= Zb. Filled from `tables/manifest.json` at boot.
const shipped = new Map();

/// Refusal codes the engine's provenance gate returns, in plain words. The numbers are
/// `PROVENANCE_REFUSED + variant` from `lib.rs`; they are listed rather than computed so a
/// new variant shows up here as "unknown refusal" instead of silently reading as another.
const REFUSALS = {
  14: "no route in this engine produces that pair's curve",
  15: "the bank is full — it holds three species at a time",
  16: "that curve does not say which solver produced it, so nothing can grade it",
  17: "that curve was produced by DMRG and presented as EXACT in the model, which it is not",
  18: "that curve was produced by DMRG, and gate D1's validation of the bridge is not recorded",
  19: "that shipped table declares no uncertainty; an absent bound must not read as a zero one",
  20: "that DMRG curve declares no convergence-derived uncertainty",
  21: "that curve is on the wrong side of the declared in-browser cost limits",
  22: "there is no usable curve in that slot",
  23: "that curve's declared uncertainty is larger than the shallowest well the schema recognises, so it cannot tell a bond from no bond",
  24: "that curve's declared uncertainty is larger than the well it claims to have found, so the well is inside its own error bar",
};

function refusalText(code) {
  return REFUSALS[code] ?? `unknown refusal (code ${code})`;
}

async function loadShippedTables() {
  try {
    const response = await fetch("tables/manifest.json");
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const manifest = await response.json();
    for (const entry of manifest.pairs) {
      const [za, zb] = entry.Z.slice().sort((a, b) => a - b);
      shipped.set(`${za}:${zb}`, entry);
    }
    state.shippedKnots = manifest.knots;
  } catch (error) {
    // Not fatal: the light pairs are still solved here, and saying which pairs are now
    // unavailable is better than a swatch that fails when clicked.
    state.shippedError = error.message;
  }
}

/// Push one shipped table into a bank slot, provenance and all.
///
/// The engine's `holon_bank_table_finish` takes the route, the counts and the uncertainty
/// as ARGUMENTS: there is no way to load a shipped curve without saying what produced it,
/// which is the point. A file that omits any of them is refused rather than defaulted.
async function loadShippedPair(w, entry) {
  const response = await fetch(`tables/${entry.file}`);
  if (!response.ok) throw new Error(`HTTP ${response.status}`);
  const file = await response.json();
  const slot = w.holon_bank_slot(entry.Z[0], entry.Z[1]);
  if (slot < 0) throw new Error("neither species is registered with the bank");
  const r = file.R_grid_bohr;
  if (!w.holon_bank_table_begin(slot, r.length)) throw new Error("the interpolator refused the grid");
  for (let i = 0; i < r.length; i += 1) {
    if (!w.holon_bank_table_knot(slot, i, r[i], file.E_hartree[i], file.F_hartree_per_bohr[i])) {
      throw new Error(`knot ${i} refused`);
    }
    if (file.E2_hartree_per_bohr2) {
      w.holon_bank_table_knot_curvature(slot, i, file.E2_hartree_per_bohr2[i]);
    }
  }
  // 1 = determinant/FCI, 2 = DMRG. Anything else is undeclared and the engine refuses it.
  const route = file.solver_route === "determinant" ? 1 : file.solver_route === "DMRG" ? 2 : 0;
  const status = w.holon_bank_table_finish(
    slot,
    file.bound ? file.R_e : 0,
    file.bound ? file.D_e : 0,
    file.E_asymptote,
    route,
    file.species.n_determinants,
    file.species.n_basis,
    file.uncertainty_hartree,
    file.exact_in_model ? 1 : 0,
  );
  if (status !== 1) throw new Error(refusalText(status));
  // Keep the file's prose for the panel: the producer line and the grid rule are strings,
  // the engine holds numbers, and this is the half the page is responsible for.
  entry.provenance = file.provenance;
  entry.grid_rule = file.grid_rule;
  entry.file_data = { R_e: file.R_e, D_e: file.D_e, n_grid: r.length };
}

/// Every species pair the scene now forms, as [Za, Zb] with Za <= Zb.
function activePairs(w) {
  const zs = new Set();
  for (let i = 0; i < w.holon_atom_count(); i += 1) zs.add(w.holon_atom_species_z(i));
  const list = [...zs].sort((a, b) => a - b);
  const out = [];
  for (let i = 0; i < list.length; i += 1) {
    for (let j = i; j < list.length; j += 1) {
      // A homonuclear pair is only formed if TWO atoms carry that species.
      if (i === j) {
        let n = 0;
        for (let k = 0; k < w.holon_atom_count(); k += 1) {
          if (w.holon_atom_species_z(k) === list[i]) n += 1;
        }
        if (n < 2) continue;
      }
      out.push([list[i], list[j]]);
    }
  }
  return out;
}

/// Make sure every pair the scene forms has a curve. Light pairs are solved here; heavy
/// ones are fetched. Returns a list of human-readable problems, empty when all is well.
async function ensureCurves(w) {
  const problems = [];
  for (const [za, zb] of activePairs(w)) {
    const slot = w.holon_bank_slot(za, zb);
    if (slot >= 0 && w.holon_bank_filled(slot)) continue;
    const key = `${za}:${zb}`;
    if (w.holon_bank_pair_route(za, zb) === 0) {
      problems.push(`${symbolOf(za)}–${symbolOf(zb)}: ${refusalText(14)}`);
      continue;
    }
    if (w.holon_bank_pair_is_heavy(za, zb) === 1) {
      const entry = shipped.get(key);
      if (!entry) {
        problems.push(
          `${symbolOf(za)}–${symbolOf(zb)} is too expensive to solve at load `
          + `(${w.holon_bank_pair_n_basis(za, zb)} basis functions, `
          + `${w.holon_bank_pair_n_det(za, zb).toLocaleString()} determinants) and no shipped `
          + `table was found${state.shippedError ? ` (${state.shippedError})` : ""}`,
        );
        continue;
      }
      try {
        await loadShippedPair(w, entry);
      } catch (error) {
        problems.push(`${symbolOf(za)}–${symbolOf(zb)}: ${error.message}`);
      }
      continue;
    }
    const status = w.holon_bank_generate_pair(za, zb, 96);
    if (status !== 1) problems.push(`${symbolOf(za)}–${symbolOf(zb)}: ${refusalText(status)}`);
  }
  return problems;
}

function symbolOf(z) {
  return state.byZ.get(z)?.symbol ?? `Z=${z}`;
}

/// Put the selected element on atom `index`, then make the bank whole again.
async function paintSpecies(w, index) {
  const z = state.selectedZ;
  if (w.holon_atom_species_z(index) === z) return;
  if (!w.holon_set_atom_species(index, z)) {
    showRefusal(`${symbolOf(z)} was refused: ${refusalText(15)}`);
    return;
  }
  const problems = await ensureCurves(w);
  if (problems.length > 0) showRefusal(problems.join("; "));
  else hideRefusal();
  // The clocks are a function of the curves, so a changed scene re-derives them.
  w.holon_rebase();
  refreshBank();
  buildPaletteAvailability(w);
}

function showRefusal(text) {
  document.querySelector("#bank-refusal").hidden = false;
  ui["bank-refusal-note"].textContent = text;
}

function hideRefusal() {
  document.querySelector("#bank-refusal").hidden = true;
}

/// Grey out the elements this scene cannot take: the bank is full, or the pair with a
/// species already present has no route, or it is heavy and nothing was shipped for it.
function buildPaletteAvailability(w) {
  const present = new Set();
  for (let i = 0; i < w.holon_atom_count(); i += 1) present.add(w.holon_atom_species_z(i));
  const full = w.holon_bank_species_count() >= w.holon_bank_max_species();
  for (const button of document.querySelectorAll(".swatch")) {
    const z = Number(button.dataset.z);
    let ok = true;
    let why = "";
    if (full && !present.has(z)) {
      ok = false;
      why = `the bank holds ${w.holon_bank_max_species()} species at a time`;
    } else {
      for (const other of [...present, z]) {
        const [a, b] = z <= other ? [z, other] : [other, z];
        if (w.holon_bank_pair_route(a, b) === 0) {
          ok = false;
          why = `${symbolOf(a)}–${symbolOf(b)} is past every solver this engine has`;
          break;
        }
        if (w.holon_bank_pair_is_heavy(a, b) === 1 && !shipped.has(`${a}:${b}`)) {
          ok = false;
          why = `${symbolOf(a)}–${symbolOf(b)} is too expensive to solve at load and is not shipped`;
          break;
        }
      }
    }
    button.dataset.available = String(ok);
    button.disabled = !ok;
    if (!ok) button.title = `${button.title} — unavailable: ${why}`;
  }
}

/// Draw the bank: one row per loaded curve, plus the fence and gate D1's record.
function refreshBank() {
  const w = state.w;
  if (!state.speciesAware) return;
  const body = ui["bank-rows"];
  body.textContent = "";
  const slots = w.holon_bank_slot_count();
  const species = [];
  for (let i = 0; i < w.holon_bank_species_count(); i += 1) species.push(w.holon_bank_species_z(i));
  let rows = 0;
  for (let i = 0; i < species.length; i += 1) {
    for (let j = i; j < species.length; j += 1) {
      const slot = w.holon_bank_slot(species[i], species[j]);
      if (slot < 0 || slot >= slots || !w.holon_bank_filled(slot)) continue;
      rows += 1;
      const routeCode = w.holon_bank_route(slot);
      const routeName = ["undeclared", "FCI (determinant)", "DMRG"][routeCode] ?? "undeclared";
      const source = w.holon_bank_source(slot) === 1 ? "shipped file" : "solved here";
      const key = `${Math.min(species[i], species[j])}:${Math.max(species[i], species[j])}`;
      const entry = shipped.get(key);
      const tr = document.createElement("tr");
      const cells = [
        [`${symbolOf(species[i])}–${symbolOf(species[j])}`, false],
        [routeName, false],
        [source, false],
        [String(w.holon_bank_pair_n_basis(species[i], species[j])), true],
        [w.holon_bank_n_det(slot).toLocaleString(), true],
        [w.holon_bank_uncertainty(slot).toExponential(3), true],
        [String(w.holon_bank_knots(slot)), true],
        [w.holon_bank_d_e(slot) > 0 ? w.holon_bank_r_e(slot).toFixed(4) : "—", true],
        [w.holon_bank_d_e(slot) > 0 ? w.holon_bank_d_e(slot).toExponential(4) : "does not bind", true],
      ];
      for (const [text, numeric] of cells) {
        const td = document.createElement("td");
        td.textContent = text;
        if (numeric) td.className = "num";
        tr.append(td);
      }
      if (routeCode === 2) tr.classList.add("route-dmrg");
      if (entry) tr.title = `${entry.provenance} — grid rule: ${entry.grid_rule}`;
      body.append(tr);
    }
  }
  ui["bank-summary"].textContent =
    `${rows} curve${rows === 1 ? "" : "s"} loaded, ${w.holon_bank_species_count()} of `
    + `${w.holon_bank_max_species()} species`;

  // THE FENCE, read from the engine rather than written here.
  ui["fence-note"].textContent = w.holon_trimer_h_only()
    ? "The tabulated three-body term is H3 ONLY. A triple containing any non-hydrogen atom "
      + "contributes exactly zero, so nothing on this page is beyond-pair-complete for such "
      + "a triple. Heteronuclear trimer surfaces are a named successor."
    : "The three-body term covers every triple.";

  const anyShipped = [...shipped.values()].some((e) => e.provenance);
  ui["bank-rule"].textContent = anyShipped
    ? [...shipped.values()]
        .filter((e) => e.provenance)
        .map((e) => `${e.pair}: ${e.provenance} · grid rule — ${e.grid_rule}`)
        .join("  ")
    : "Every curve above was solved in this browser at load, from Z and the declared "
      + "STO-3G basis. Pairs too expensive to solve here arrive as shipped tables and "
      + "state their producer, grid rule and uncertainty when they do.";

  const d1 = w.holon_d1_validated() === 1;
  ui["d1-note"].textContent = d1
    ? `Gate D1: the DMRG bridge is ADMITTED — worst overlap `
      + `${w.holon_d1_worst_overlap().toExponential(3)} Ha against a stake of `
      + `${w.holon_d1_stake().toExponential(0)} Ha on ${w.holon_d1_overlap_species()} species. `
      + `A DMRG curve may enter the bank, labelled DMRG, never as exact.`
    : "Gate D1: the DMRG bridge is NOT admitted — its validation is not recorded. Every "
      + "DMRG-labelled curve is refused by the provenance gate, so every curve above was "
      + "produced by the determinant route.";
}

/// The colour and radius one atom is drawn with.
///
/// Reads the atom's species from the wasm when the ABI offers it and falls back to
/// hydrogen when it does not, so the day `holon_atom_z` appears this becomes correct with
/// no edit here.
function atomStyle(w, i) {
  const z = state.speciesAware ? w.holon_atom_species_z(i) : 1;
  const sp = state.byZ.get(z);
  if (!sp) return { colour: "#236957", scale: 1 };
  const h = state.byZ.get(1);
  return { colour: sp.colour, scale: h ? sp.radius_bohr / h.radius_bohr : 1 };
}

async function boot() {
  let instance;
  const response = await fetch("holon_render.wasm");
  try {
    instance = (await WebAssembly.instantiateStreaming(response.clone(), {})).instance;
  } catch {
    instance = (await WebAssembly.instantiate(await response.arrayBuffer(), {})).instance;
  }
  const w = instance.exports;
  state.w = w;
  // FEATURE DETECTION, against the exports that actually exist.
  //
  // This read `holon_atom_z` / `holon_set_atom_z` for as long as the palette has been on
  // the page, and the engine has never exported either — the names are
  // `holon_atom_species_z` and `holon_set_atom_species`. So `speciesAware` was false for
  // reasons that had nothing to do with whether the engine could carry a species, and the
  // "SELECTOR ONLY" banner was telling the truth for the wrong reason. It is now gated on
  // the bank, which is the thing that actually decides whether a species can be put in the
  // scene: a sim with one table has nowhere to put the second pair's curve.
  state.speciesAware = typeof w.holon_atom_species_z === "function"
    && typeof w.holon_set_atom_species === "function"
    && typeof w.holon_bank_generate_pair === "function";
  await loadPotential(w);
  loadTrimer(w);
  await loadPalette();
  if (!state.speciesAware) {
    document.querySelector("#spawn-banner").hidden = false;
    ui["spawn-note"].textContent =
      "This build's engine carries one potential table, so it has nowhere to record what "
      + "an atom IS. The palette still reports what the engine computed about each "
      + "element; every atom on the stage is hydrogen, and is drawn as hydrogen.";
  } else {
    await loadShippedTables();
    refreshBank();
  }

  state.world = { width: w.holon_width(), height: w.holon_height() };
  state.radius = w.holon_wall_inset();
  state.baseSimSpeed = w.holon_sim_speed();

  calibrate(w);

  applyControls();
  w.holon_reset(Number(controls.atomCount.value));

  clampAtomSlider();
  document.body.dataset.engine = "ready";
  ui["runtime-status"].textContent = `Rust/WASM live · ${w.holon_substeps_per_second().toExponential(1)} substeps/s`;
  requestAnimationFrame(frame);
}

/// Measure THIS device, rather than assuming it.
///
/// "How many atoms on low-end mobile" is not answerable from a developer's laptop, so the
/// page finds out on load: a burst of pure physics at the maximum atom count, no
/// rendering, timed here. The wasm runs the substeps and this side holds the clock,
/// because `std::time` does not exist on wasm32-unknown-unknown and a second timing path
/// for native would be a second thing to keep true.
///
/// The burst is sized in TIME rather than in steps: it ramps until it has spent about
/// 200 ms, so a slow device is not punished with a long stall and a fast one still gets a
/// sample long enough to be worth trusting.
function calibrate(w) {
  const targetMs = 200;
  let substeps = 2000;
  let elapsed = 0;
  let total = 0;
  const t0 = performance.now();
  // Discard the first burst: it pays for warm-up (JIT, page faults, cache) that the
  // steady-state rate should not be charged for.
  w.holon_calibration_burst(500);
  while (performance.now() - t0 < targetMs) {
    const a = performance.now();
    w.holon_calibration_burst(substeps);
    const b = performance.now();
    elapsed += b - a;
    total += substeps;
    if (b - a < 20) substeps *= 2;
  }
  if (elapsed <= 0) return;
  w.holon_set_calibration((total / elapsed) * 1000);
}

// ---------------------------------------------------------------- controls

function applyControls() {
  const w = state.w;
  w.holon_set_boundary(controls.openBoundary.checked ? 1 : 0);
  w.holon_set_thermostat(controls.thermostat.checked ? 1 : 0, Number(controls.targetT.value));
  w.holon_set_census_enabled(controls.censusEnabled.checked ? 1 : 0);

  // The sim-speed slider is logarithmic around the derived default, because the useful
  // range spans a couple of decades and a linear slider would spend all its travel at the
  // fast end.
  const speed = state.baseSimSpeed * Math.pow(10, Number(controls.simSpeed.value));
  w.holon_set_sim_speed(speed);

  // Rung (ii) is behind an explicit toggle, and the slider that grows dt is inert without
  // it. Turning the toggle off re-derives dt from the envelope immediately.
  w.holon_set_allow_dt_growth(controls.allowDtGrowth.checked ? 1 : 0);
  controls.dtMult.disabled = !controls.allowDtGrowth.checked;
  if (controls.allowDtGrowth.checked) {
    w.holon_set_dt_multiplier(Number(controls.dtMult.value));
  }

  ui["atom-count-out"].textContent = controls.atomCount.value;
  ui["target-t-out"].textContent = controls.targetT.value;
  ui["dt-mult-out"].textContent = Number(controls.dtMult.value).toFixed(1);
  ui["sim-speed-out"].textContent = speed.toFixed(speed < 10 ? 2 : 0);
}

for (const el of [
  controls.simSpeed, controls.targetT, controls.thermostat,
  controls.allowDtGrowth, controls.dtMult, controls.censusEnabled,
]) {
  el.addEventListener("input", applyControls);
}
// Changing the atom count or the boundary changes the SCENE, so the ledger has to be
// re-based: comparing against an origin taken before a different scene existed would
// report a drift that no integrator produced.
for (const el of [controls.atomCount, controls.openBoundary]) {
  el.addEventListener("input", () => {
    applyControls();
    state.w.holon_reset(Number(controls.atomCount.value));
  });
}

/// Clamp the atom slider to what this device was MEASURED to sustain.
function clampAtomSlider() {
  const nmax = Math.max(2, Math.min(16, Math.floor(state.w.holon_n_max())));
  controls.atomCount.max = String(nmax);
  if (Number(controls.atomCount.value) > nmax) {
    controls.atomCount.value = String(nmax);
    ui["atom-count-out"].textContent = String(nmax);
  }
}
controls.reset.addEventListener("click", () => {
  applyControls();
  state.w.holon_reset(Number(controls.atomCount.value));
});

// ---------------------------------------------------------------- pointer

function worldFromEvent(event) {
  const rect = stage.getBoundingClientRect();
  const scale = state.world.width / rect.width;
  return {
    x: (event.clientX - rect.left) * scale,
    y: (event.clientY - rect.top) * scale,
  };
}

stage.addEventListener("pointerdown", (event) => {
  const p = worldFromEvent(event);
  // A generous pick radius: the atoms are small on screen and the grab is meant to feel
  // like reaching for one, not like hitting a target.
  const index = state.w.holon_nearest_atom(p.x, p.y, 2.5 * state.radius);
  if (index < 0) return;
  // SHIFT (or ALT) PAINTS THE SELECTED ELEMENT; a plain press still grabs and drags.
  //
  // A modifier rather than a mode, because the two actions want the same gesture on the
  // same target and a mode would mean the same click did different things depending on
  // state the user cannot see. Painting is async — a heavy pair has to be fetched, a
  // light one solved — so it deliberately does not start a drag.
  if (state.speciesAware && (event.shiftKey || event.altKey)) {
    event.preventDefault();
    void paintSpecies(state.w, index);
    return;
  }
  state.w.holon_grab(index);
  state.dragging = true;
  // Pointer capture is a convenience (it keeps the drag alive when the pointer leaves
  // the canvas), not a requirement, and it throws for a pointerId the browser does not
  // consider active. Losing the capture must not lose the grab.
  try {
    stage.setPointerCapture(event.pointerId);
  } catch {
    /* drag continues without capture */
  }
});

stage.addEventListener("pointermove", (event) => {
  if (!state.dragging) return;
  const p = worldFromEvent(event);
  state.w.holon_move_anchor(p.x, p.y);
});

for (const type of ["pointerup", "pointercancel"]) {
  stage.addEventListener(type, () => {
    if (!state.dragging) return;
    state.dragging = false;
    state.w.holon_release();
  });
}

// ---------------------------------------------------------------- rendering

/// Mix a hex colour towards white. Used only for the highlight side of an atom's
/// gradient, so the palette needs to declare ONE colour per element rather than two.
function lighten(hex, t) {
  const n = parseInt(hex.slice(1), 16);
  const mix = (c) => Math.round(c + (255 - c) * t);
  return `rgb(${mix((n >> 16) & 255)}, ${mix((n >> 8) & 255)}, ${mix(n & 255)})`;
}

function fitCanvas(canvas, context, aspect) {
  const ratio = Math.min(window.devicePixelRatio || 1, 2);
  const rect = canvas.getBoundingClientRect();
  const width = Math.max(1, Math.round(rect.width * ratio));
  const height = Math.max(1, Math.round((aspect ? rect.width / aspect : rect.height) * ratio));
  if (canvas.width !== width || canvas.height !== height) {
    canvas.width = width;
    canvas.height = height;
  }
  context.setTransform(ratio, 0, 0, ratio, 0, 0);
  return { width: rect.width, height: height / ratio };
}

function drawScene() {
  const w = state.w;
  const view = fitCanvas(stage, ctx, state.world.width / state.world.height);
  const s = view.width / state.world.width;
  ctx.clearRect(0, 0, view.width, view.height);

  // The wall, where the physics actually puts it.
  if (!controls.openBoundary.checked) {
    const inset = w.holon_wall_inset() * s;
    ctx.strokeStyle = "rgba(27, 42, 36, 0.20)";
    ctx.setLineDash([5, 5]);
    ctx.lineWidth = 1;
    ctx.strokeRect(inset, inset, view.width - 2 * inset, view.height - 2 * inset);
    ctx.setLineDash([]);
  }

  // Molecules first, so the atoms sit on top of them.
  //
  // Drawn from the composite-holon ROWS, not from the pair predicate. A molecule is an
  // entry in a table with its own ledger and a formation time, and the picture shows that
  // table -- so what is on screen and what the census counts cannot disagree.
  const rows = w.holon_row_count();
  for (let k = 0; k < rows; k += 1) {
    const i = w.holon_row_member(k, 0);
    const j = w.holon_row_member(k, 1);
    const depth = Math.min(1, -w.holon_row_e_bond(k) / Math.max(1e-12, w.holon_table_d_e()));
    ctx.strokeStyle = `rgba(35, 105, 87, ${0.35 + 0.55 * depth})`;
    ctx.lineWidth = 2 + 5 * depth;
    ctx.lineCap = "round";
    ctx.beginPath();
    ctx.moveTo(w.holon_atom_x(i) * s, w.holon_atom_y(i) * s);
    ctx.lineTo(w.holon_atom_x(j) * s, w.holon_atom_y(j) * s);
    ctx.stroke();
  }

  // The user's spring, drawn from the atom to the pointer anchor.
  const grabbed = w.holon_grabbed();
  if (grabbed >= 0) {
    ctx.strokeStyle = "rgba(187, 76, 46, 0.65)";
    ctx.lineWidth = 1.5;
    ctx.setLineDash([3, 4]);
    ctx.beginPath();
    ctx.moveTo(w.holon_atom_x(grabbed) * s, w.holon_atom_y(grabbed) * s);
    ctx.lineTo(w.holon_anchor_x() * s, w.holon_anchor_y() * s);
    ctx.stroke();
    ctx.setLineDash([]);
  }

  const n = w.holon_atom_count();
  for (let i = 0; i < n; i += 1) {
    const x = w.holon_atom_x(i) * s;
    const y = w.holon_atom_y(i) * s;
    // The drawn radius scales with the SPECIES, in the ratio the engine derived, about
    // the wall inset the physics already uses — so the picture and the boundary keep
    // agreeing about where an atom's edge is.
    const style = atomStyle(w, i);
    const r = state.radius * style.scale * s;
    const gradient = ctx.createRadialGradient(x - r * 0.3, y - r * 0.35, r * 0.15, x, y, r);
    const held = i === grabbed;
    gradient.addColorStop(0, held ? "#e8a08a" : lighten(style.colour, 0.42));
    gradient.addColorStop(1, held ? "#bb4c2e" : style.colour);
    ctx.fillStyle = gradient;
    ctx.beginPath();
    ctx.arc(x, y, r, 0, Math.PI * 2);
    ctx.fill();
  }
}

// The curve inset. Drawn by asking the wasm for U(R) point by point, so it is literally
// the function the integrator differentiates — not a JS copy that could drift from it.
function drawCurve() {
  const w = state.w;
  const view = fitCanvas(curveCanvas, curveCtx, null);
  const W = view.width, H = view.height;
  curveCtx.clearRect(0, 0, W, H);
  const rMin = 0.6, rMax = 9.0;
  const de = w.holon_table_d_e();
  const uMin = -1.15 * de, uMax = 0.45 * de;
  const px = (r) => ((r - rMin) / (rMax - rMin)) * W;
  const py = (u) => H - ((u - uMin) / (uMax - uMin)) * H;

  // Zero line: the dissociation asymptote, which is the bond criterion's threshold.
  curveCtx.strokeStyle = "rgba(27, 42, 36, 0.25)";
  curveCtx.lineWidth = 1;
  curveCtx.setLineDash([4, 4]);
  curveCtx.beginPath();
  curveCtx.moveTo(0, py(0));
  curveCtx.lineTo(W, py(0));
  curveCtx.stroke();
  curveCtx.setLineDash([]);

  curveCtx.strokeStyle = "#236957";
  curveCtx.lineWidth = 1.8;
  curveCtx.beginPath();
  for (let i = 0; i <= 240; i += 1) {
    const r = rMin + (rMax - rMin) * (i / 240);
    const u = Math.max(uMin, Math.min(uMax, w.holon_curve_u(r)));
    if (i === 0) curveCtx.moveTo(px(r), py(u));
    else curveCtx.lineTo(px(r), py(u));
  }
  curveCtx.stroke();

  // The tightest pair, and why it reads the way it does.
  const pairs = w.holon_pair_count();
  let best = -1, bestR = Infinity;
  for (let k = 0; k < pairs; k += 1) {
    const r = w.holon_pair_r(k);
    if (r < bestR) { bestR = r; best = k; }
  }
  if (best < 0) return;
  const eRel = w.holon_pair_e_rel(best);
  const rOuter = w.holon_pair_r_outer(best);
  const bonded = w.holon_pair_bonded(best) === 1;

  // E_rel as a level: where it sits relative to the dashed asymptote IS the criterion.
  if (eRel > uMin && eRel < uMax) {
    curveCtx.strokeStyle = bonded ? "rgba(35, 105, 87, 0.75)" : "rgba(187, 76, 46, 0.75)";
    curveCtx.lineWidth = 1.2;
    curveCtx.beginPath();
    curveCtx.moveTo(0, py(eRel));
    curveCtx.lineTo(W, py(eRel));
    curveCtx.stroke();
  }
  if (Number.isFinite(rOuter) && rOuter > rMin && rOuter < rMax) {
    curveCtx.strokeStyle = "rgba(212, 154, 54, 0.9)";
    curveCtx.setLineDash([2, 3]);
    curveCtx.beginPath();
    curveCtx.moveTo(px(rOuter), 0);
    curveCtx.lineTo(px(rOuter), H);
    curveCtx.stroke();
    curveCtx.setLineDash([]);
  }
  if (bestR > rMin && bestR < rMax) {
    const u = Math.max(uMin, Math.min(uMax, w.holon_curve_u(bestR)));
    curveCtx.fillStyle = bonded ? "#236957" : "#bb4c2e";
    curveCtx.beginPath();
    curveCtx.arc(px(bestR), py(u), 4, 0, Math.PI * 2);
    curveCtx.fill();
  }

  ui["pair-r"].textContent = `${bestR.toFixed(4)} a0`;
  ui["pair-erel"].textContent = `${eRel.toExponential(3)} Eh`;
  ui["pair-router"].textContent = Number.isFinite(rOuter) ? `${rOuter.toFixed(3)} a0` : "unbound";
}

const eh = (v) => (Math.abs(v) < 1e-4 && v !== 0 ? v.toExponential(3) : v.toFixed(6));

function drawLedger() {
  const w = state.w;
  ui["e-kin"].textContent = eh(w.holon_e_kin());
  ui["e-pair"].textContent = eh(w.holon_e_pair());
  ui["e-three"].textContent = eh(w.holon_e_three ? w.holon_e_three() : 0);
  ui["e-wall"].textContent = eh(w.holon_e_wall());
  ui["e-spring"].textContent = eh(w.holon_e_spring());
  ui["w-ext"].textContent = eh(w.holon_w_ext());
  ui["ledger"].textContent = eh(w.holon_ledger());

  const drift = w.holon_drift_peak();
  const bound = w.holon_drift_bound();
  const pass = w.holon_energy_gate() === 1;
  ui["drift"].textContent = drift.toExponential(3);
  ui["drift-bound"].textContent = bound.toExponential(3);
  const ratio = bound > 0 ? drift / bound : 0;
  ui["drift-ratio"].textContent = `${(100 * ratio).toFixed(1)} %`;
  ui["drift-fill"].style.width = `${Math.min(100, 100 * ratio)}%`;
  ui["drift-fill"].dataset.state = pass ? "pass" : "fail";
  ui["energy-gate"].textContent = pass ? "PASS" : "FAIL";
  ui["energy-gate"].dataset.state = pass ? "pass" : "fail";

  const px = w.holon_momentum_x(), pyv = w.holon_momentum_y();
  const mPass = w.holon_momentum_gate() === 1;
  ui["p-mag"].textContent = Math.hypot(px, pyv).toExponential(4);
  ui["p-res"].textContent = w.holon_momentum_residual_peak().toExponential(3);
  ui["p-bound"].textContent = w.holon_momentum_bound().toExponential(3);
  ui["momentum-gate"].textContent = mPass ? "PASS" : "FAIL";
  ui["momentum-gate"].dataset.state = mPass ? "pass" : "fail";

  const bonded = w.holon_bonded_count();
  ui["bond-count"].textContent = bonded;
  ui["temperature"].textContent = `${w.holon_temperature().toFixed(0)} K`;
  ui["clock"].textContent = `t = ${w.holon_time().toFixed(0)} a.u.`;
  // The headline names the CLUSTER, not the pair: on a collapsed droplet every
  // pair reads mutually bound (16 atoms -> 120 bonded pairs, all of them true
  // two-body statements), so the pair count is a diagnostic, not a headline.
  const clusters = w.holon_cluster_count();
  const clusterAtoms = w.holon_cluster_atoms();
  ui["stage-status"].textContent =
    bonded === 0 ? "NO BOND"
    : clusters === 1 && clusterAtoms === 2 ? "BONDED"
    : `${clusters} CLUSTER${clusters > 1 ? "S" : ""} · ${clusterAtoms} ATOMS`;
}

const RUNGS = [
  ["EXACT", "pass", "accuracy held, sim-speed delivered"],
  ["DILATED", "dilated", "rung (i): time dilates, accuracy untouched"],
  ["DECLARED", "declared", "rung (ii): dt grown, bound re-derived below"],
  ["REFUSED", "refused", "omega*dt reached the stability limit"],
];

function drawClocks() {
  const w = state.w;
  const [label, cls, note] = RUNGS[w.holon_rung()] ?? RUNGS[0];
  ui.rung.textContent = label;
  ui.rung.dataset.state = cls;
  ui["rung-note"].textContent = note;

  ui.dt.textContent = `${w.holon_dt().toFixed(4)} a.u.`;
  ui["dt-ref"].textContent = `${w.holon_dt_reference().toFixed(4)} a.u.`;
  ui["omega-e"].textContent = w.holon_omega_e().toExponential(3);
  ui["omega-env"].textContent = w.holon_omega_env().toExponential(3);
  ui["omega-dt"].textContent = w.holon_omega_dt().toFixed(4);
  ui["frame-ms"].textContent = `${state.frameMs.toFixed(1)} ms`;
  ui["substeps-frame"].textContent = state.substepsThisFrame;
  const speed = w.holon_sim_speed();
  ui["sim-speed"].textContent = `${speed.toFixed(speed < 10 ? 2 : 0)} fs/s`;
  const periodFs = w.holon_period_fs();
  ui["vib-wall"].textContent = `${(periodFs / Math.max(1e-12, speed)).toFixed(2)} s`;
  ui.dilation.textContent = `${(100 * w.holon_dilation()).toFixed(0)} %`;

  ui["c-atoms"].textContent = w.holon_census_atoms();
  ui["c-molecules"].textContent = w.holon_census_molecules();
  ui["c-candidates"].textContent = w.holon_census_candidates();
  ui["c-global"].textContent = w.holon_census_global_views();
  ui["c-formations"].textContent = w.holon_census_formations();
  ui["c-dissolutions"].textContent = w.holon_census_dissolutions();
  ui["c-rejections"].textContent = w.holon_census_closure_rejections();
  ui["c-bondsector"].textContent = eh(w.holon_bond_sector_energy());

  ui["d-sps"].textContent = w.holon_substeps_per_second().toExponential(2);
  ui["d-pps"].textContent = w.holon_pairs_per_second().toExponential(2);
  ui["d-req"].textContent = w.holon_required_substeps_per_second().toExponential(2);
  ui["d-nmax"].textContent = w.holon_calibrated() ? Math.floor(w.holon_n_max()) : "—";
}

function frame(now) {
  // Clock 2, MEASURED. The first frame has no predecessor to measure against, so it
  // advances nothing rather than guessing an interval.
  const wallDt = state.lastFrameMs === null ? 0 : (now - state.lastFrameMs) / 1000;
  state.lastFrameMs = now;
  state.frameMs = wallDt * 1000;

  state.substepsThisFrame = state.w.holon_advance_frame(wallDt);

  drawScene();
  drawCurve();
  drawLedger();
  drawClocks();
  requestAnimationFrame(frame);
}

boot().catch((error) => {
  document.body.dataset.engine = "failed";
  ui["runtime-status"].textContent = "refused";
  showError(String(error && error.message ? error.message : error));
});
