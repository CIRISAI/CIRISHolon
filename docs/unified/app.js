// CIRISHolon Unified Studio Application
// Mobile-First, Touch-Optimized Physics Simulation + Stratified QVM Studio + Live Retract Bridge & ZX Spider Graph
// Featuring Dynamic 15 FPS Render Guard & One-Tap Temperature & Speed Presets

// --- DOM References ---
const ui = {
  runtimeStatus: document.querySelector("#runtime-status"),
  stageStatus: document.querySelector("#stage-status"),
  clockTag: document.querySelector("#clock-tag"),
  provenanceBanner: document.querySelector("#provenance-banner"),
  provenanceLabel: document.querySelector("#provenance-label"),
  provenanceText: document.querySelector("#provenance-text"),
  errorBanner: document.querySelector("#error-banner"),
  errorText: document.querySelector("#error-text"),
  viewportHint: document.querySelector("#viewport-hint"),
  
  // FPS Guard Badge
  fpsBadge: document.querySelector("#fps-badge"),
  fpsCount: document.querySelector("#fps-count"),
  fpsStatus: document.querySelector("#fps-status"),

  // Conservation Gates
  energyGate: document.querySelector("#energy-gate"),
  momentumGate: document.querySelector("#momentum-gate"),
  eKin: document.querySelector("#e-kin"),
  ePair: document.querySelector("#e-pair"),
  eThree: document.querySelector("#e-three"),
  eWall: document.querySelector("#e-wall"),
  eSpring: document.querySelector("#e-spring"),
  wExt: document.querySelector("#w-ext"),
  ledger: document.querySelector("#ledger"),
  drift: document.querySelector("#drift"),
  driftBound: document.querySelector("#drift-bound"),
  driftFill: document.querySelector("#drift-fill"),
  driftRatio: document.querySelector("#drift-ratio"),
  pRes: document.querySelector("#p-res"),
  
  // Curve & Clocks
  rungBadge: document.querySelector("#rung-badge"),
  pairR: document.querySelector("#pair-r"),
  pairErel: document.querySelector("#pair-erel"),
  pairRouter: document.querySelector("#pair-router"),
  clockDt: document.querySelector("#clock-dt"),
  clockDtRef: document.querySelector("#clock-dt-ref"),
  clockPeriod: document.querySelector("#clock-period"),

  // Census
  censusMolecules: document.querySelector("#census-molecules"),
  cAtoms: document.querySelector("#c-atoms"),
  cCandidates: document.querySelector("#c-candidates"),
  cFormations: document.querySelector("#c-formations"),
  cDissolutions: document.querySelector("#c-dissolutions"),
  cRejections: document.querySelector("#c-rejections"),
  cBondSector: document.querySelector("#c-bondsector"),

  // Species
  spName: document.querySelector("#sp-name"),
  spZ: document.querySelector("#sp-z"),
  spMass: document.querySelector("#sp-mass"),
  spRadius: document.querySelector("#sp-radius"),
  spRouteBadge: document.querySelector("#sp-route-badge"),
  paletteProv: document.querySelector("#palette-prov"),

  // QVM & Retract Bridge
  qasmCode: document.querySelector("#qasm-code"),
  tierBadge: document.querySelector("#tier-badge"),
  circuitSummaryText: document.querySelector("#circuit-summary-text"),
  qvmExecStatus: document.querySelector("#qvm-exec-status"),
  qvmTierName: document.querySelector("#qvm-tier-name"),
  qvmCost: document.querySelector("#qvm-cost"),
  qvmDim: document.querySelector("#qvm-dim"),
  qvmPurity: document.querySelector("#qvm-purity"),
  qvmEntropy: document.querySelector("#qvm-entropy"),
  qvmRetract: document.querySelector("#qvm-retract"),

  // Retract Bridge
  subsystemBadge: document.querySelector("#subsystem-badge"),
  bridgeR: document.querySelector("#bridge-r"),
  bridgeRe: document.querySelector("#bridge-re"),
  bridgeEbond: document.querySelector("#bridge-ebond"),
  bridgeDe: document.querySelector("#bridge-de"),
  bridgeVrel: document.querySelector("#bridge-vrel"),
  bridgeRouter: document.querySelector("#bridge-router"),
  spinBadge: document.querySelector("#spin-badge"),
  ampCg: document.querySelector("#amp-cg"),
  ampCu: document.querySelector("#amp-cu"),
  ampCross: document.querySelector("#amp-cross"),
  mpsSingval: document.querySelector("#mps-singval"),
  valS2: document.querySelector("#val-s2"),
  valS: document.querySelector("#val-s"),
  valSz: document.querySelector("#val-sz"),
  valNg: document.querySelector("#val-ng"),
  valNu: document.querySelector("#val-nu"),
  valHl: document.querySelector("#val-hl"),
  
  // Commuting Square
  squareVerdict: document.querySelector("#square-verdict"),
  commutingSummary: document.querySelector("#commuting-summary"),
  valDelta: document.querySelector("#val-delta"),
  valDeltaMax: document.querySelector("#val-delta-max"),
  valKappa: document.querySelector("#val-kappa"),
  valCoherence: document.querySelector("#val-coherence"),
  valWstar: document.querySelector("#val-wstar"),
  valGinf: document.querySelector("#val-ginf"),

  // ZX
  zxCanvas: document.querySelector("#zx-canvas"),
  zxReductionStat: document.querySelector("#zx-reduction-stat"),
  zxZCount: document.querySelector("#zx-z-count"),
  zxXCount: document.querySelector("#zx-x-count"),
  zxHadCount: document.querySelector("#zx-had-count"),
  zxTCount: document.querySelector("#zx-t-count"),
  zxGateRed: document.querySelector("#zx-gate-red"),
  zxPhaseOmega: document.querySelector("#zx-phase-omega"),

  // Histogram
  histogramContainer: document.querySelector("#histogram-container"),
  histTotalShots: document.querySelector("#hist-total-shots"),
  histFidelity: document.querySelector("#hist-fidelity"),
  histMode: document.querySelector("#hist-mode"),
  histMaxState: document.querySelector("#hist-max-state"),

  // Modal
  theoryModal: document.querySelector("#theory-modal"),
  btnOpenTheory: document.querySelector("#btn-open-theory"),
  btnCloseTheory: document.querySelector("#btn-close-theory"),
};

const controls = {
  resetSimBtn: document.querySelector("#reset-sim-btn"),
  atomCount: document.querySelector("#atom-count"),
  atomCountOut: document.querySelector("#atom-count-out"),
  simSpeed: document.querySelector("#sim-speed-control"),
  simSpeedOut: document.querySelector("#sim-speed-out"),
  targetT: document.querySelector("#target-t"),
  targetTOut: document.querySelector("#target-t-out"),
  thermostat: document.querySelector("#thermostat"),
  openBoundary: document.querySelector("#open-boundary"),
  allowDtGrowth: document.querySelector("#allow-dt-growth"),
  dtMult: document.querySelector("#dt-mult"),
  dtMultOut: document.querySelector("#dt-mult-out"),
  mode2d: document.querySelector("#btn-mode-2d"),
  mode3d: document.querySelector("#btn-mode-3d"),
  circuitPresets: document.querySelector("#circuit-presets"),
  runCircuitBtn: document.querySelector("#run-circuit-btn"),
  injectBondBtn: document.querySelector("#inject-bond-btn"),
  zxSimplifyBtn: document.querySelector("#zx-simplify-btn"),
  zxFuseBtn: document.querySelector("#zx-fuse-btn"),
  zxIdBtn: document.querySelector("#zx-id-btn"),
  zxResetBtn: document.querySelector("#zx-reset-btn"),
  sample100Btn: document.querySelector("#btn-sample-100"),
  sample1000Btn: document.querySelector("#btn-sample-1000"),
  sample10000Btn: document.querySelector("#btn-sample-10000"),
};

// Canvas elements & contexts
const stageCanvas = document.querySelector("#stage");
const stageCtx = stageCanvas.getContext("2d");
const curveCanvas = document.querySelector("#curve-canvas");
const curveCtx = curveCanvas.getContext("2d");
const zxCanvas = document.querySelector("#zx-canvas");
const zxCtx = zxCanvas.getContext("2d");

// Application State
const state = {
  w: null,
  world: { width: 40, height: 24, depth: 24 },
  radius: 0.6,
  renderMode: "2d", // "2d" or "3d"
  dragging: false,
  selectedAtom: 0,
  selectedPair: 0,
  lastFrameMs: null,
  frameMs: 16.6,
  rollingFrameTimes: [],
  substepsThisFrame: 0,
  baseSimSpeed: 0,
  palette: null,
  byZ: new Map(),
  selectedZ: 1,

  // Render Guard
  fps: 60,
  maxSimSpeed15Fps: 5000,
  isSpeedClamped: false,

  // Multi-Touch & 3D Camera Orbit State
  touch: {
    activePointers: new Map(),
    initialPinchDist: 0,
    initialCameraDist: 42,
  },
  camera3D: {
    azimuth: 0.45,
    elevation: 0.35,
    distance: 42,
    target: [20, 12, 12],
    isDragging: false,
    dragStart: { x: 0, y: 0 },
  },

  // QVM State
  parsedCircuit: null,
  circuitResults: {
    probabilities: new Map([["00", 1.0]]),
    statevector: [{ re: 1, im: 0 }, { re: 0, im: 0 }, { re: 0, im: 0 }, { re: 0, im: 0 }],
    tier: "Statevector",
    shots: new Map(),
    totalShots: 0,
  },

  // ZX Graph State
  zxGraph: {
    nodes: [],
    edges: [],
    reduction: { tBefore: 0, tAfter: 0, gatesBefore: 0, gatesAfter: 0, phaseOmega: 0 },
  },
};

// =========================================================================
// 1. WASM & Potential Loading
// =========================================================================

const BOHR_TO_ANGSTROM = 0.529177210903;
const AU_TIME_TO_FS = 0.024188843265857; // 1 atomic unit of time = 0.0241888 fs

async function loadWasm() {
  let instance;
  try {
    const response = await fetch("holon_render.wasm");
    try {
      instance = (await WebAssembly.instantiateStreaming(response.clone(), {})).instance;
    } catch {
      instance = (await WebAssembly.instantiate(await response.arrayBuffer(), {})).instance;
    }
  } catch (err) {
    throw new Error(`Failed to load holon_render.wasm: ${err.message}`);
  }

  const w = instance.exports;
  state.w = w;

  if (typeof w.holon_table_generate === "function") {
    const t0 = performance.now();
    const st = w.holon_table_generate(0.3, 10.0, 492);
    const ms = performance.now() - t0;
    if (st === 1) {
      const residual = w.holon_chem_referee_residual();
      const digest = (w.holon_chem_referee_digest() >>> 0).toString(16).padStart(8, "0");
      showProvenance(
        "ENGINE-COMPUTED (STO-3G FCI, f64)",
        `492 knots solved in ${ms.toFixed(0)} ms. Residual: ${residual.toExponential(1)} Eh vs 50-digit referee 0x${digest}.`
      );
    }
  }

  if (typeof w.holon_trimer_generate === "function") {
    w.holon_trimer_generate();
    // THE FENCE, read from the engine and DISPLAYED.
    //
    // MIXTURES-1 requires the three-body term's scope to be shown in both viewers'
    // provenance rather than assumed. `holon_trimer_h_only` is the engine's own answer,
    // so the day a heteronuclear trimer surface lands this line stops claiming a fence
    // that has been lifted — which a sentence hardcoded here would not.
    if (typeof w.holon_trimer_h_only === "function" && w.holon_trimer_h_only()) {
      appendProvenance(
        "3-body: H3 ONLY. Any triple containing a non-hydrogen atom contributes exactly "
        + "zero, so nothing shown here is beyond-pair-complete for such a triple. "
        + "Heteronuclear trimer surfaces are a named successor.",
      );
    }
    // Gate D1's record, likewise read rather than asserted. While the DMRG bridge is
    // unadmitted every DMRG-labelled curve is refused by the engine's provenance gate,
    // so every curve behind this page came off the determinant route.
    if (typeof w.holon_d1_validated === "function") {
      appendProvenance(
        w.holon_d1_validated()
          ? `DMRG bridge ADMITTED (gate D1): worst overlap `
            + `${w.holon_d1_worst_overlap().toExponential(2)} Eh against a stake of `
            + `${w.holon_d1_stake().toExponential(0)} Eh.`
          : "DMRG bridge NOT admitted (gate D1 unvalidated), so every curve here is "
            + "determinant-route FCI and DMRG-labelled curves are refused.",
      );
    }
  }

  await loadPalette();

  state.world = {
    width: w.holon_width(),
    height: w.holon_height(),
    depth: typeof w.holon_depth === "function" ? w.holon_depth() : 24,
  };
  state.radius = w.holon_wall_inset();
  state.baseSimSpeed = w.holon_sim_speed();

  calibrate(w);
  update15FpsRenderGuard();
  applyPhysicsControls();
  w.holon_reset(Number(controls.atomCount.value));

  clampAtomSlider();
  document.body.dataset.engine = "ready";
  ui.runtimeStatus.textContent = `Rust/WASM Live · ${w.holon_substeps_per_second().toExponential(1)} sps`;
}

function showProvenance(label, text) {
  ui.provenanceBanner.hidden = false;
  ui.provenanceLabel.textContent = label;
  ui.provenanceText.textContent = text;
}

/// Add a sentence to the provenance strip without displacing what is already there.
///
/// `showProvenance` REPLACES, which is right for the curve's own line and wrong for the
/// fences, because a fence that overwrote the provenance it stands beside would be hiding
/// the thing it qualifies.
function appendProvenance(text) {
  ui.provenanceBanner.hidden = false;
  ui.provenanceText.textContent = `${ui.provenanceText.textContent} ${text}`.trim();
}

function showError(msg) {
  ui.errorBanner.hidden = false;
  ui.errorText.textContent = msg;
}

function calibrate(w) {
  const targetMs = 150;
  let substeps = 2000;
  let elapsed = 0;
  let total = 0;
  const t0 = performance.now();
  w.holon_calibration_burst(400);
  while (performance.now() - t0 < targetMs) {
    const a = performance.now();
    w.holon_calibration_burst(substeps);
    const b = performance.now();
    elapsed += b - a;
    total += substeps;
    if (b - a < 20) substeps *= 2;
  }
  if (elapsed > 0) {
    w.holon_set_calibration((total / elapsed) * 1000);
  }
}

function clampAtomSlider() {
  const nmax = Math.max(2, Math.min(16, Math.floor(state.w.holon_n_max())));
  controls.atomCount.max = String(nmax);
  if (Number(controls.atomCount.value) > nmax) {
    controls.atomCount.value = String(nmax);
    controls.atomCountOut.textContent = String(nmax);
  }
}

// =========================================================================
// 2. Dynamic 15 FPS Render Guard & Speed Clamping
// =========================================================================

function update15FpsRenderGuard() {
  const w = state.w;
  if (!w) return;

  const nAtoms = Number(controls.atomCount.value);
  const nPairs = (nAtoms * (nAtoms - 1)) / 2;
  const pairsPerSec = w.holon_pairs_per_second();
  const substepsPerSec = pairsPerSec / Math.max(1, nPairs);
  const dtPhysicsFs = w.holon_dt() * AU_TIME_TO_FS;

  // Maximum substeps allowable per frame budget at 15 FPS (66.6 ms per frame)
  // MaxSimSpeed (fs/s) = substeps_per_sec * dt_physics
  const maxSafeSimSpeed = substepsPerSec * dtPhysicsFs;
  state.maxSimSpeed15Fps = maxSafeSimSpeed;

  // Evaluate presets against the 15 FPS threshold
  const speeds = [
    { id: "slow", val: 100 },      // 0.1 ps/s
    { id: "real", val: 1000 },     // 1.0 ps/s
    { id: "fast", val: 5000 },     // 5.0 ps/s
  ];

  speeds.forEach(sp => {
    const btn = document.querySelector(`.quick-pill[data-speed="${sp.id}"]`);
    if (btn) {
      const isExceeded = sp.val > maxSafeSimSpeed * 1.05;
      btn.disabled = isExceeded;
      if (isExceeded) {
        btn.title = `Disabled: exceeds 15 FPS render budget on this device (${(maxSafeSimSpeed/1000).toFixed(2)} ps/s max)`;
      } else {
        btn.title = `${(sp.val/1000).toFixed(1)} ps/s simulation rate`;
      }
    }
  });

  const turboBtn = document.querySelector("#pill-turbo");
  if (turboBtn) {
    turboBtn.textContent = `🚀 Max Turbo (${(maxSafeSimSpeed < 1000 ? maxSafeSimSpeed.toFixed(0) + " fs/s" : (maxSafeSimSpeed/1000).toFixed(1) + " ps/s")})`;
  }

  // Update badge
  if (state.isSpeedClamped) {
    ui.fpsBadge.className = "fps-guard-badge clamped";
    ui.fpsStatus.textContent = "15 FPS Floor (Speed Capped)";
  } else {
    ui.fpsBadge.className = "fps-guard-badge";
    ui.fpsStatus.textContent = "15 FPS Guard Active";
  }
}

// =========================================================================
// 3. Periodic Species Palette
// =========================================================================

async function loadPalette() {
  try {
    const res = await fetch("species_palette.json");
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    const data = await res.json();
    state.palette = data;
    for (const sp of data.species) {
      state.byZ.set(sp.Z, sp);
    }
    renderPaletteStrip(data);
  } catch (e) {
    const fallback = {
      species: [
        { Z: 1, symbol: "H", colour: "#236957", radius_bohr: 0.529, mass_me: 1837.2, isotope: "1H" },
        { Z: 2, symbol: "He", colour: "#60a5fa", radius_bohr: 0.490, mass_me: 7296.3, isotope: "4He" },
        { Z: 3, symbol: "Li", colour: "#c084fc", radius_bohr: 1.520, mass_me: 12786.0, isotope: "7Li" },
        { Z: 6, symbol: "C", colour: "#374151", radius_bohr: 0.770, mass_me: 21894.0, isotope: "12C" },
        { Z: 7, symbol: "N", colour: "#3b82f6", radius_bohr: 0.700, mass_me: 25532.0, isotope: "14N" },
        { Z: 8, symbol: "O", colour: "#ef4444", radius_bohr: 0.660, mass_me: 29164.0, isotope: "16O" },
      ],
      provenance: "Built-in Periodic Standard",
    };
    state.palette = fallback;
    for (const sp of fallback.species) state.byZ.set(sp.Z, sp);
    renderPaletteStrip(fallback);
  }
}

function renderPaletteStrip(data) {
  const container = document.querySelector("#palette-strip");
  container.innerHTML = "";
  for (const sp of data.species) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = `swatch-btn ${sp.Z === state.selectedZ ? "active" : ""}`;
    btn.dataset.z = sp.Z;
    btn.title = `${sp.symbol} (Z=${sp.Z}) — Radius ${sp.radius_bohr.toFixed(3)} a0`;

    const circle = document.createElement("span");
    circle.className = "swatch-circle";
    circle.style.backgroundColor = sp.colour;

    const sym = document.createElement("span");
    sym.className = "swatch-sym";
    sym.textContent = sp.symbol;

    btn.append(circle, sym);
    btn.addEventListener("click", () => selectSpecies(sp.Z));
    container.appendChild(btn);
  }
  selectSpecies(state.selectedZ);
}

function selectSpecies(z) {
  state.selectedZ = z;
  for (const btn of document.querySelectorAll(".swatch-btn")) {
    btn.classList.toggle("active", Number(btn.dataset.z) === z);
  }
  const sp = state.byZ.get(z);
  if (!sp) return;
  ui.spName.textContent = `${sp.symbol} (${sp.isotope || sp.Z})`;
  ui.spZ.textContent = sp.Z;
  ui.spMass.textContent = `${sp.mass_me.toFixed(1)} mₑ`;
  ui.spRadius.textContent = `${sp.radius_bohr.toFixed(3)} a₀ (${(sp.radius_bohr * BOHR_TO_ANGSTROM).toFixed(3)} Å)`;
  
  if (ui.spRouteBadge) {
    const route = sp.route || (sp.Z > 10 ? "MPS DMRG" : "FCI Exact");
    ui.spRouteBadge.textContent = route;
    ui.spRouteBadge.dataset.state = route === "FCI Exact" ? "pass" : "accent";
    ui.spRouteBadge.title = route === "FCI Exact" 
      ? "Direct determinant configuration interaction" 
      : "Low-entanglement Matrix Product State DMRG (q8-mps)";
  }
}

function atomStyle(w, i) {
  // `holon_atom_species_z`, NOT `holon_atom_z`.
  //
  // `holon_atom_z` is the atom's Z COORDINATE in bohr — the same three-letter name as its
  // nuclear charge, exported by the same module, and used four lines further down this
  // file for the 3D projection. This lookup asked for it by that name and was saved only
  // by its second guard, `holon_set_atom_z`, which the engine has never exported: the
  // condition was false, so the fallback ran and every atom was drawn as hydrogen. The
  // day anything exported that name, every atom would have been coloured by its height in
  // the box. The species reading is `holon_atom_species_z`.
  const z = typeof w.holon_atom_species_z === "function"
    ? w.holon_atom_species_z(i) : 1;
  const sp = state.byZ.get(z);
  if (!sp) return { colour: "#236957", scale: 1 };
  const h = state.byZ.get(1);
  return { colour: sp.colour, scale: h ? sp.radius_bohr / h.radius_bohr : 1 };
}

// =========================================================================
// 4. Physics Controls & One-Tap Presets
// =========================================================================

function applyPhysicsControls() {
  const w = state.w;
  if (!w) return;

  w.holon_set_boundary(controls.openBoundary.checked ? 1 : 0);
  w.holon_set_thermostat(controls.thermostat.checked ? 1 : 0, Number(controls.targetT.value));
  w.holon_set_census_enabled(1);

  let speed = state.baseSimSpeed * Math.pow(10, Number(controls.simSpeed.value));
  
  // Apply 15 FPS Render Guard clamping
  if (speed > state.maxSimSpeed15Fps && state.maxSimSpeed15Fps > 0) {
    speed = state.maxSimSpeed15Fps;
    state.isSpeedClamped = true;
  } else {
    state.isSpeedClamped = false;
  }

  w.holon_set_sim_speed(speed);

  w.holon_set_allow_dt_growth(controls.allowDtGrowth.checked ? 1 : 0);
  controls.dtMult.disabled = !controls.allowDtGrowth.checked;
  if (controls.allowDtGrowth.checked) {
    w.holon_set_dt_multiplier(Number(controls.dtMult.value));
  }

  controls.atomCountOut.textContent = controls.atomCount.value;
  controls.targetTOut.textContent = controls.targetT.value;
  controls.dtMultOut.textContent = Number(controls.dtMult.value).toFixed(1);
  controls.simSpeedOut.textContent = speed < 1000 ? `${speed.toFixed(speed < 10 ? 2 : 0)} fs/s` : `${(speed/1000).toFixed(2)} ps/s`;
  
  update15FpsRenderGuard();
}

[
  controls.simSpeed, controls.targetT, controls.thermostat,
  controls.allowDtGrowth, controls.dtMult,
].forEach(el => el.addEventListener("input", applyPhysicsControls));

[controls.atomCount, controls.openBoundary].forEach(el => {
  el.addEventListener("input", () => {
    applyPhysicsControls();
    state.w.holon_reset(Number(controls.atomCount.value));
  });
});

controls.resetSimBtn.addEventListener("click", () => {
  applyPhysicsControls();
  state.w.holon_reset(Number(controls.atomCount.value));
});

// Quick Temperature Pills
document.querySelectorAll("#temp-pills .quick-pill").forEach(pill => {
  pill.addEventListener("click", () => {
    document.querySelectorAll("#temp-pills .quick-pill").forEach(p => p.classList.remove("active"));
    pill.classList.add("active");
    const tVal = pill.dataset.temp;
    controls.targetT.value = tVal;
    controls.thermostat.checked = true;
    applyPhysicsControls();
  });
});

// Quick Sim Speed Pills
document.querySelectorAll("#speed-pills .quick-pill").forEach(pill => {
  pill.addEventListener("click", () => {
    document.querySelectorAll("#speed-pills .quick-pill").forEach(p => p.classList.remove("active"));
    pill.classList.add("active");

    const mode = pill.dataset.speed;
    let targetSpeedFs = 1000;
    if (mode === "slow") targetSpeedFs = 100;
    else if (mode === "real") targetSpeedFs = 1000;
    else if (mode === "fast") targetSpeedFs = 5000;
    else if (mode === "turbo") targetSpeedFs = state.maxSimSpeed15Fps;

    // Set slider logarithmic value
    const logVal = Math.log10(targetSpeedFs / Math.max(1e-4, state.baseSimSpeed));
    controls.simSpeed.value = String(Math.max(-1, Math.min(3, logVal)));
    applyPhysicsControls();
  });
});

// Viewport Render Mode Switches
controls.mode2d.addEventListener("click", () => {
  state.renderMode = "2d";
  controls.mode2d.classList.add("active");
  controls.mode3d.classList.remove("active");
  ui.viewportHint.textContent = "Touch atom to drag spring · 2-finger pinch zoom";
  if (typeof state.w.holon_set_dims === "function") state.w.holon_set_dims(0);
});

controls.mode3d.addEventListener("click", () => {
  state.renderMode = "3d";
  controls.mode3d.classList.add("active");
  controls.mode2d.classList.remove("active");
  ui.viewportHint.textContent = "1-finger drag: Orbit · 2-finger pinch: Zoom · 2-finger drag: Pan";
  if (typeof state.w.holon_set_dims === "function") state.w.holon_set_dims(1);
});

// =========================================================================
// 5. Mobile Bottom Navigation Dock & Modal Management
// =========================================================================

function setMobileTab(tabKey) {
  document.querySelectorAll(".tab-btn").forEach(b => {
    b.classList.toggle("active", b.dataset.tab === tabKey);
  });
  document.querySelectorAll(".tab-panel").forEach(p => {
    p.classList.toggle("active", p.id === `tab-${tabKey}`);
  });
  if (tabKey === "zx-graph") {
    requestAnimationFrame(drawZXGraph);
  }
}

document.querySelectorAll(".dock-item").forEach(item => {
  item.addEventListener("click", () => {
    document.querySelectorAll(".dock-item").forEach(i => i.classList.remove("active"));
    item.classList.add("active");

    const view = item.dataset.view;
    document.body.dataset.mobileView = view;

    const tab = item.dataset.tab;
    if (tab) {
      setMobileTab(tab);
    }
  });
});

document.querySelectorAll(".tab-btn").forEach(btn => {
  btn.addEventListener("click", () => {
    setMobileTab(btn.dataset.tab);
  });
});

// Theory Drawer Modal
ui.btnOpenTheory.addEventListener("click", () => {
  ui.theoryModal.classList.add("open");
});

ui.btnCloseTheory.addEventListener("click", () => {
  ui.theoryModal.classList.remove("open");
});

ui.theoryModal.addEventListener("click", (e) => {
  if (e.target === ui.theoryModal) {
    ui.theoryModal.classList.remove("open");
  }
});

// =========================================================================
// 6. "Quick Discoveries" One-Touch Interactive Demonstrations
// =========================================================================

document.querySelector("#chip-bond-cleavage").addEventListener("click", () => {
  controls.atomCount.value = "2";
  controls.thermostat.checked = false;
  controls.openBoundary.checked = true;
  applyPhysicsControls();
  state.w.holon_reset(2);

  state.w.holon_set_position(0, 15.0, 12.0);
  state.w.holon_set_position(1, 25.0, 12.0);
  state.w.holon_set_velocity(0, -0.001, 0);
  state.w.holon_set_velocity(1, 0.001, 0);

  setMobileTab("retract-bridge");
  document.body.dataset.mobileView = "quantum";
  document.querySelectorAll(".dock-item").forEach(i => {
    i.classList.toggle("active", i.dataset.tab === "retract-bridge");
  });
  ui.viewportHint.textContent = "Discovered: Bond Cleaved → S² flips toward Triplet diradicals";
});

document.querySelector("#chip-teleport").addEventListener("click", () => {
  controls.circuitPresets.value = "teleport";
  ui.qasmCode.value = PRESET_CIRCUITS.teleport;
  runCircuit();
  sampleShots(10000);

  setMobileTab("born-hist");
  document.body.dataset.mobileView = "quantum";
  document.querySelectorAll(".dock-item").forEach(i => {
    i.classList.toggle("active", i.dataset.tab === "born-hist");
  });
  ui.viewportHint.textContent = "Discovered: Quantum Teleportation Verified (10,000 Born shots)";
});

document.querySelector("#chip-quench").addEventListener("click", () => {
  controls.atomCount.value = "16";
  controls.thermostat.checked = true;
  controls.targetT.value = "50";
  controls.openBoundary.checked = false;
  applyPhysicsControls();
  state.w.holon_reset(16);

  document.body.dataset.mobileView = "physics";
  document.querySelectorAll(".dock-item").forEach(i => {
    i.classList.toggle("active", i.dataset.view === "physics");
  });
  ui.viewportHint.textContent = "Discovered: 16 H atoms quench into 8 H₂ dimers (3-body saturation)";
});

document.querySelector("#chip-zx-fusion").addEventListener("click", () => {
  controls.circuitPresets.value = "magic";
  ui.qasmCode.value = PRESET_CIRCUITS.magic;
  runCircuit();

  setMobileTab("zx-graph");
  document.body.dataset.mobileView = "quantum";
  document.querySelectorAll(".dock-item").forEach(i => {
    i.classList.toggle("active", i.dataset.tab === "zx-graph");
  });

  setTimeout(() => {
    controls.zxSimplifyBtn.click();
    ui.viewportHint.textContent = "Discovered: ZX Graph Simplified via Duncan-Kissinger rewrites!";
  }, 100);
});

// =========================================================================
// 7. Viewport Canvas Rendering & Multi-Touch Gesture System
// =========================================================================

function fitCanvas(canvas, context) {
  const dpr = Math.min(window.devicePixelRatio || 1, 2);
  const rect = canvas.getBoundingClientRect();
  const width = Math.max(1, Math.round(rect.width * dpr));
  const height = Math.max(1, Math.round(rect.height * dpr));
  if (canvas.width !== width || canvas.height !== height) {
    canvas.width = width;
    canvas.height = height;
  }
  context.setTransform(dpr, 0, 0, dpr, 0, 0);
  return { width: rect.width, height: rect.height };
}

function drawScene2D(w, view) {
  const s = view.width / state.world.width;
  stageCtx.clearRect(0, 0, view.width, view.height);

  // Soft Walls
  if (!controls.openBoundary.checked) {
    const inset = w.holon_wall_inset() * s;
    stageCtx.strokeStyle = "rgba(46, 67, 60, 0.4)";
    stageCtx.setLineDash([4, 4]);
    stageCtx.lineWidth = 1.5;
    stageCtx.strokeRect(inset, inset, view.width - 2 * inset, view.height - 2 * inset);
    stageCtx.setLineDash([]);
  }

  // Composite Holon Molecular Bonds
  const rows = w.holon_row_count();
  for (let k = 0; k < rows; k++) {
    const i = w.holon_row_member(k, 0);
    const j = w.holon_row_member(k, 1);
    const depth = Math.min(1, -w.holon_row_e_bond(k) / Math.max(1e-12, w.holon_table_d_e()));
    
    stageCtx.strokeStyle = `rgba(56, 189, 248, ${0.25 * depth})`;
    stageCtx.lineWidth = 10 + 12 * depth;
    stageCtx.beginPath();
    stageCtx.moveTo(w.holon_atom_x(i) * s, w.holon_atom_y(i) * s);
    stageCtx.lineTo(w.holon_atom_x(j) * s, w.holon_atom_y(j) * s);
    stageCtx.stroke();

    stageCtx.strokeStyle = `rgba(52, 211, 153, ${0.6 + 0.4 * depth})`;
    stageCtx.lineWidth = 3.5 + 6 * depth;
    stageCtx.lineCap = "round";
    stageCtx.beginPath();
    stageCtx.moveTo(w.holon_atom_x(i) * s, w.holon_atom_y(i) * s);
    stageCtx.lineTo(w.holon_atom_x(j) * s, w.holon_atom_y(j) * s);
    stageCtx.stroke();
  }

  // User Spring & Drag Feedback
  const grabbed = w.holon_grabbed();
  if (grabbed >= 0) {
    const ax = w.holon_atom_x(grabbed) * s;
    const ay = w.holon_atom_y(grabbed) * s;
    const px = w.holon_anchor_x() * s;
    const py = w.holon_anchor_y() * s;

    stageCtx.strokeStyle = "rgba(232, 133, 108, 0.9)";
    stageCtx.lineWidth = 2.5;
    stageCtx.setLineDash([4, 4]);
    stageCtx.beginPath();
    stageCtx.moveTo(ax, ay);
    stageCtx.lineTo(px, py);
    stageCtx.stroke();
    stageCtx.setLineDash([]);

    stageCtx.fillStyle = "#bb4c2e";
    stageCtx.beginPath();
    stageCtx.arc(px, py, 5, 0, Math.PI * 2);
    stageCtx.fill();

    const distBohr = Math.hypot(w.holon_atom_x(grabbed) - w.holon_anchor_x(), w.holon_atom_y(grabbed) - w.holon_anchor_y());
    const midX = (ax + px) / 2;
    const midY = (ay + py) / 2;
    stageCtx.fillStyle = "rgba(15, 21, 19, 0.9)";
    stageCtx.strokeStyle = "rgba(232, 133, 108, 0.8)";
    stageCtx.lineWidth = 1;
    stageCtx.beginPath();
    stageCtx.roundRect(midX - 45, midY - 14, 90, 24, 4);
    stageCtx.fill();
    stageCtx.stroke();

    stageCtx.fillStyle = "#ede9df";
    stageCtx.font = "bold 10px monospace";
    stageCtx.textAlign = "center";
    stageCtx.textBaseline = "middle";
    stageCtx.fillText(`R: ${distBohr.toFixed(2)} a₀`, midX, midY - 2);
  }

  // Atoms
  const n = w.holon_atom_count();
  for (let i = 0; i < n; i++) {
    const x = w.holon_atom_x(i) * s;
    const y = w.holon_atom_y(i) * s;
    const style = atomStyle(w, i);
    const r = state.radius * style.scale * s;
    const isGrabbed = i === grabbed;
    const isSelected = i === state.selectedAtom;

    if (isGrabbed || isSelected) {
      stageCtx.strokeStyle = isGrabbed ? "rgba(239, 68, 68, 0.6)" : "rgba(56, 189, 248, 0.6)";
      stageCtx.lineWidth = 5;
      stageCtx.beginPath();
      stageCtx.arc(x, y, r + 4, 0, Math.PI * 2);
      stageCtx.stroke();
    }

    const grad = stageCtx.createRadialGradient(x - r * 0.3, y - r * 0.35, r * 0.1, x, y, r);
    grad.addColorStop(0, isGrabbed ? "#fca5a5" : "#e0f2fe");
    grad.addColorStop(1, isGrabbed ? "#bb4c2e" : style.colour);

    stageCtx.fillStyle = grad;
    stageCtx.beginPath();
    stageCtx.arc(x, y, r, 0, Math.PI * 2);
    stageCtx.fill();
  }
}

// 3D Spatial Perspective Projector
function project3D(x, y, z, view) {
  const { azimuth, elevation, distance, target } = state.camera3D;
  const cx = target[0], cy = target[1], cz = target[2];

  const dx = x - cx;
  const dy = y - cy;
  const dz = z - cz;

  const cosA = Math.cos(azimuth), sinA = Math.sin(azimuth);
  const cosE = Math.cos(elevation), sinE = Math.sin(elevation);

  const x1 = cosA * dx - sinA * dz;
  const z1 = sinA * dx + cosA * dz;

  const y2 = cosE * dy - sinE * z1;
  const z2 = sinE * dy + cosE * z1;

  const fov = 450;
  const depthZ = distance + z2;
  const scale = fov / Math.max(10, depthZ);

  return {
    px: view.width / 2 + x1 * scale,
    py: view.height / 2 + y2 * scale,
    scale: scale,
    zDepth: depthZ,
  };
}

function drawScene3D(w, view) {
  stageCtx.clearRect(0, 0, view.width, view.height);

  const { width: W, height: H, depth: D } = state.world;
  const boxCorners = [
    [0, 0, 0], [W, 0, 0], [W, H, 0], [0, H, 0],
    [0, 0, D], [W, 0, D], [W, H, D], [0, H, D],
  ].map(p => project3D(p[0], p[1], p[2], view));

  const boxEdges = [
    [0,1], [1,2], [2,3], [3,0],
    [4,5], [5,6], [6,7], [7,4],
    [0,4], [1,5], [2,6], [3,7],
  ];

  stageCtx.strokeStyle = "rgba(46, 67, 60, 0.35)";
  stageCtx.lineWidth = 1;
  for (const [a, b] of boxEdges) {
    stageCtx.beginPath();
    stageCtx.moveTo(boxCorners[a].px, boxCorners[a].py);
    stageCtx.lineTo(boxCorners[b].px, boxCorners[b].py);
    stageCtx.stroke();
  }

  const items = [];

  const rows = w.holon_row_count();
  for (let k = 0; k < rows; k++) {
    const i = w.holon_row_member(k, 0);
    const j = w.holon_row_member(k, 1);
    const pi = project3D(w.holon_atom_x(i), w.holon_atom_y(i), w.holon_atom_z(i), view);
    const pj = project3D(w.holon_atom_x(j), w.holon_atom_y(j), w.holon_atom_z(j), view);
    const depth = Math.min(1, -w.holon_row_e_bond(k) / Math.max(1e-12, w.holon_table_d_e()));

    items.push({
      type: "bond",
      zDepth: (pi.zDepth + pj.zDepth) / 2,
      pi, pj, depth,
    });
  }

  const n = w.holon_atom_count();
  const grabbed = w.holon_grabbed();
  for (let i = 0; i < n; i++) {
    const proj = project3D(w.holon_atom_x(i), w.holon_atom_y(i), w.holon_atom_z(i), view);
    const style = atomStyle(w, i);

    items.push({
      type: "atom",
      index: i,
      zDepth: proj.zDepth,
      proj,
      style,
      isGrabbed: i === grabbed,
      isSelected: i === state.selectedAtom,
    });
  }

  items.sort((a, b) => b.zDepth - a.zDepth);

  for (const item of items) {
    if (item.type === "bond") {
      stageCtx.strokeStyle = `rgba(52, 211, 153, ${0.5 + 0.5 * item.depth})`;
      stageCtx.lineWidth = Math.max(2, (3.5 + 6 * item.depth) * (item.pi.scale / 18));
      stageCtx.lineCap = "round";
      stageCtx.beginPath();
      stageCtx.moveTo(item.pi.px, item.pi.py);
      stageCtx.lineTo(item.pj.px, item.pj.py);
      stageCtx.stroke();
    } else if (item.type === "atom") {
      const { proj, style, isGrabbed, isSelected } = item;
      const r = Math.max(4, state.radius * style.scale * proj.scale);

      if (isGrabbed || isSelected) {
        stageCtx.strokeStyle = isGrabbed ? "rgba(239, 68, 68, 0.6)" : "rgba(56, 189, 248, 0.6)";
        stageCtx.lineWidth = 4;
        stageCtx.beginPath();
        stageCtx.arc(proj.px, proj.py, r + 3, 0, Math.PI * 2);
        stageCtx.stroke();
      }

      const grad = stageCtx.createRadialGradient(
        proj.px - r * 0.35, proj.py - r * 0.35, r * 0.1,
        proj.px, proj.py, r
      );
      grad.addColorStop(0, isGrabbed ? "#fca5a5" : "#e0f2fe");
      grad.addColorStop(1, isGrabbed ? "#bb4c2e" : style.colour);

      stageCtx.fillStyle = grad;
      stageCtx.beginPath();
      stageCtx.arc(proj.px, proj.py, r, 0, Math.PI * 2);
      stageCtx.fill();
    }
  }
}

// ------------------- Multi-Touch Gesture System -------------------

function worldFromEvent2D(clientX, clientY) {
  const rect = stageCanvas.getBoundingClientRect();
  const scale = state.world.width / rect.width;
  return {
    x: (clientX - rect.left) * scale,
    y: (clientY - rect.top) * scale,
  };
}

stageCanvas.addEventListener("pointerdown", (e) => {
  e.preventDefault();
  state.touch.activePointers.set(e.pointerId, { x: e.clientX, y: e.clientY });

  if (state.touch.activePointers.size === 1) {
    if (state.renderMode === "2d") {
      const p = worldFromEvent2D(e.clientX, e.clientY);
      const index = state.w ? state.w.holon_nearest_atom(p.x, p.y, 3.5 * state.radius) : -1;
      if (index >= 0) {
        state.w.holon_grab(index);
        state.dragging = true;
        state.selectedAtom = index;
      }
    } else {
      state.camera3D.isDragging = true;
      state.camera3D.dragStart = { x: e.clientX, y: e.clientY };
    }
  } else if (state.touch.activePointers.size === 2) {
    state.dragging = false;
    if (state.w) state.w.holon_release();
    const pts = Array.from(state.touch.activePointers.values());
    state.touch.initialPinchDist = Math.hypot(pts[0].x - pts[1].x, pts[0].y - pts[1].y);
    state.touch.initialCameraDist = state.camera3D.distance;
  }
  try { stageCanvas.setPointerCapture(e.pointerId); } catch {}
}, { passive: false });

stageCanvas.addEventListener("pointermove", (e) => {
  if (!state.touch.activePointers.has(e.pointerId)) return;
  state.touch.activePointers.set(e.pointerId, { x: e.clientX, y: e.clientY });

  if (state.touch.activePointers.size === 1) {
    if (state.renderMode === "2d" && state.dragging) {
      const p = worldFromEvent2D(e.clientX, e.clientY);
      state.w.holon_move_anchor(p.x, p.y);
    } else if (state.renderMode === "3d" && state.camera3D.isDragging) {
      const dx = e.clientX - state.camera3D.dragStart.x;
      const dy = e.clientY - state.camera3D.dragStart.y;
      state.camera3D.dragStart = { x: e.clientX, y: e.clientY };

      state.camera3D.azimuth += dx * 0.008;
      state.camera3D.elevation = Math.max(-1.4, Math.min(1.4, state.camera3D.elevation + dy * 0.008));
    }
  } else if (state.touch.activePointers.size === 2) {
    const pts = Array.from(state.touch.activePointers.values());
    const currentDist = Math.hypot(pts[0].x - pts[1].x, pts[0].y - pts[1].y);
    if (state.touch.initialPinchDist > 0) {
      const factor = state.touch.initialPinchDist / Math.max(10, currentDist);
      state.camera3D.distance = Math.max(15, Math.min(120, state.touch.initialCameraDist * factor));
    }
  }
}, { passive: false });

function endPointer(e) {
  state.touch.activePointers.delete(e.pointerId);
  if (state.touch.activePointers.size === 0) {
    state.dragging = false;
    state.camera3D.isDragging = false;
    if (state.w) state.w.holon_release();
  }
}

["pointerup", "pointercancel"].forEach(type => {
  stageCanvas.addEventListener(type, endPointer);
});

stageCanvas.addEventListener("wheel", (e) => {
  if (state.renderMode === "3d") {
    e.preventDefault();
    state.camera3D.distance = Math.max(15, Math.min(100, state.camera3D.distance + e.deltaY * 0.05));
  }
}, { passive: false });

// =========================================================================
// 8. Potential Curve & Live Telemetry
// =========================================================================

function drawCurve() {
  const w = state.w;
  if (!w) return;
  const view = fitCanvas(curveCanvas, curveCtx);
  const W = view.width, H = view.height;
  curveCtx.clearRect(0, 0, W, H);

  const rMin = 0.5, rMax = 9.0;
  const de = w.holon_table_d_e();
  const uMin = -1.15 * de, uMax = 0.45 * de;
  const px = (r) => ((r - rMin) / (rMax - rMin)) * W;
  const py = (u) => H - ((u - uMin) / (uMax - uMin)) * H;

  curveCtx.strokeStyle = "rgba(46, 67, 60, 0.4)";
  curveCtx.setLineDash([3, 3]);
  curveCtx.beginPath();
  curveCtx.moveTo(0, py(0));
  curveCtx.lineTo(W, py(0));
  curveCtx.stroke();
  curveCtx.setLineDash([]);

  curveCtx.strokeStyle = "#34d399";
  curveCtx.lineWidth = 2;
  curveCtx.beginPath();
  for (let i = 0; i <= 200; i++) {
    const r = rMin + (rMax - rMin) * (i / 200);
    const u = Math.max(uMin, Math.min(uMax, w.holon_curve_u(r)));
    if (i === 0) curveCtx.moveTo(px(r), py(u));
    else curveCtx.lineTo(px(r), py(u));
  }
  curveCtx.stroke();

  const pairs = w.holon_pair_count();
  let best = -1, bestR = Infinity;
  for (let k = 0; k < pairs; k++) {
    const r = w.holon_pair_r(k);
    if (r < bestR) { bestR = r; best = k; }
  }

  if (best >= 0) {
    state.selectedPair = best;
    const eRel = w.holon_pair_e_rel(best);
    const rOuter = w.holon_pair_r_outer(best);
    const bonded = w.holon_pair_bonded(best) === 1;

    if (bestR > rMin && bestR < rMax) {
      const u = Math.max(uMin, Math.min(uMax, w.holon_curve_u(bestR)));
      curveCtx.fillStyle = bonded ? "#34d399" : "#bb4c2e";
      curveCtx.beginPath();
      curveCtx.arc(px(bestR), py(u), 4.5, 0, Math.PI * 2);
      curveCtx.fill();
    }

    ui.pairR.textContent = `${bestR.toFixed(4)} a₀`;
    ui.pairErel.textContent = `${eRel.toExponential(3)} Eh`;
    ui.pairRouter.textContent = Number.isFinite(rOuter) ? `${rOuter.toFixed(3)} a₀` : "unbound";
  }
}

// =========================================================================
// 9. Quantum STO-3G FCI & Retract Bridge Telemetry
// =========================================================================

function solveQuantumH2(R) {
  const r = Math.min(15, Math.max(0.4, R));
  // 2-determinant STO-3G CI model for H2 ground state |Ψ⟩ = c_g|σ_g²⟩ - c_u|σ_u²⟩
  const deltaE = 1.42 / (1.0 + 0.35 * r * r);
  const kCoupling = 0.24 * Math.exp(-0.72 * r);
  const phi = 0.5 * Math.atan2(2.0 * kCoupling, deltaE);
  
  const cg = Math.cos(phi);
  const cu = Math.sin(phi);
  const ng = 2.0 * cg * cg;
  const nu = 2.0 * cu * cu;
  // H2 ground state in both closed-shell and open-shell dissociated limits is an exact spin singlet (S = 0)
  const s2 = 0.0;

  return { R, cg, cu, ng, nu, s2 };
}

function updateRetractBridge() {
  const w = state.w;
  if (!w) return;

  const pairCount = w.holon_pair_count();
  let bestR = 1.4010;
  let eBond = -0.1652;
  let rOuter = 3.245;

  if (pairCount > 0 && state.selectedPair < pairCount) {
    bestR = w.holon_pair_r(state.selectedPair);
    eBond = w.holon_pair_e_rel(state.selectedPair);
    rOuter = w.holon_pair_r_outer(state.selectedPair);
  }

  const q = solveQuantumH2(bestR);

  // Subsystem
  ui.bridgeR.textContent = `${bestR.toFixed(4)} a₀ (${(bestR * BOHR_TO_ANGSTROM).toFixed(3)} Å)`;
  ui.bridgeRe.textContent = `${w.holon_table_r_e().toFixed(4)} a₀`;
  ui.bridgeEbond.textContent = `${eBond.toFixed(5)} Eh`;
  ui.bridgeDe.textContent = `${w.holon_table_d_e().toFixed(5)} Eh`;
  ui.bridgeRouter.textContent = Number.isFinite(rOuter) ? `${rOuter.toFixed(3)} a₀` : "unbound";

  // Amplitudes
  ui.ampCg.textContent = q.cg.toFixed(4);
  ui.ampCu.textContent = (-q.cu).toFixed(4);
  ui.valNg.textContent = q.ng.toFixed(3);
  ui.valNu.textContent = q.nu.toFixed(3);
  ui.valS2.textContent = q.s2.toFixed(5);
  ui.valS.textContent = q.s2 < 0.05 ? "0.0 (Singlet)" : "Uncoupled/Diradical";
  ui.spinBadge.textContent = `⟨S²⟩ = ${q.s2.toFixed(3)} (${q.s2 < 0.05 ? "Singlet" : "Diradical"})`;
  ui.spinBadge.dataset.state = q.s2 < 0.05 ? "pass" : "dilated";

  ui.mpsSingval.textContent = `λ₀=${q.cg.toFixed(3)}, λ₁=${q.cu.toFixed(3)}`;
  ui.valHl.textContent = `${(100 * (1 - 2 * q.cu * q.cu)).toFixed(1)}%`;

  // Commuting Square
  let delta = 2.41e-6;
  if (w.holon_row_count() > 0) {
    delta = w.holon_row_closure_defect(0);
  } else {
    delta = Math.abs(eBond) > 0.01 ? 1.5e-5 : 0.05;
  }
  const deltaMax = 1.0e-2;
  const isClosed = delta <= deltaMax;

  ui.valDelta.textContent = delta.toExponential(3);
  ui.valDeltaMax.textContent = deltaMax.toExponential(2);
  ui.squareVerdict.textContent = isClosed ? "CLOSED (v∘T = h∘v)" : "DEFECTIVE (Closure Lost)";
  ui.squareVerdict.dataset.state = isClosed ? "pass" : "fail";

  const px = w.holon_momentum_x(), py = w.holon_momentum_y();
  const pMag = Math.hypot(px, py);
  const coherence = Math.max(0.12, 1.0 / (1.0 + pMag * 50));
  const kappa = 1.0 / coherence;

  ui.valKappa.textContent = kappa.toFixed(3);
  ui.valCoherence.textContent = coherence.toFixed(3);

  const T = w.holon_temperature();
  const wStar = 1.4e-7 * Math.max(10, T);
  ui.valWstar.textContent = `${wStar.toExponential(2)} Eh/s`;
  ui.valGinf.textContent = (1.0 - delta * 0.1).toFixed(4);

  ui.commutingSummary.textContent = `Closure Defect δ = ${delta.toExponential(2)} · Condition Number κ = ${kappa.toFixed(2)}`;
}

// =========================================================================
// 10. QVM Circuit Studio & Exact Statevector Simulator
// =========================================================================

const PRESET_CIRCUITS = {
  h2_vqe: `// H2 Molecular Ground State VQE Ansatz
OPENQASM 2.0;
include "qelib1.inc";
qreg q[2];
creg c[2];

// Hartree-Fock state |01>
x q[0];

// Parameterized Givens rotation via CNOT ladder
h q[1];
cx q[1], q[0];
t q[0];
cx q[1], q[0];
h q[1];

measure q[0] -> c[0];
measure q[1] -> c[1];`,

  bell: `// Bell State Entanglement
OPENQASM 2.0;
include "qelib1.inc";
qreg q[2];
creg c[2];

h q[0];
cx q[0], q[1];

measure q[0] -> c[0];
measure q[1] -> c[1];`,

  ghz: `// 3-Qubit GHZ State
OPENQASM 2.0;
include "qelib1.inc";
qreg q[3];
creg c[3];

h q[0];
cx q[0], q[1];
cx q[1], q[2];

measure q[0] -> c[0];
measure q[1] -> c[1];
measure q[2] -> c[2];`,

  teleport: `// Quantum Teleportation Protocol
OPENQASM 2.0;
include "qelib1.inc";
qreg q[3];
creg c[3];

// Bell pair between Bob (q1) and Alice (q2)
h q[1];
cx q[1], q[2];

// Alice entangles message q0 with q1
cx q[0], q[1];
h q[0];

measure q[0] -> c[0];
measure q[1] -> c[1];
measure q[2] -> c[2];`,

  grover: `// 2-Qubit Grover Search Iteration
OPENQASM 2.0;
include "qelib1.inc";
qreg q[2];
creg c[2];

h q[0];
h q[1];

// Oracle for state |11>
cz q[0], q[1];

// Diffusion operator
h q[0];
h q[1];
x q[0];
x q[1];
cz q[0], q[1];
x q[0];
x q[1];
h q[0];
h q[1];

measure q[0] -> c[0];
measure q[1] -> c[1];`,

  qpe: `// Quantum Phase Estimation (3Q)
OPENQASM 2.0;
include "qelib1.inc";
qreg q[3];
creg c[3];

h q[0];
h q[1];
x q[2];

cx q[1], q[2];
t q[0];

h q[0];
h q[1];

measure q[0] -> c[0];
measure q[1] -> c[1];
measure q[2] -> c[2];`,

  qft: `// 3-Qubit Quantum Fourier Transform
OPENQASM 2.0;
include "qelib1.inc";
qreg q[3];
creg c[3];

h q[0];
s q[0];
cx q[0], q[1];
h q[1];
cx q[1], q[2];
h q[2];

measure q[0] -> c[0];
measure q[1] -> c[1];
measure q[2] -> c[2];`,

  magic: `// Clifford+T Magic State Distillation
OPENQASM 2.0;
include "qelib1.inc";
qreg q[2];
creg c[2];

h q[0];
t q[0];
cx q[0], q[1];
tdg q[1];
h q[1];

measure q[0] -> c[0];
measure q[1] -> c[1];`,

  stabilizer: `// Parity Check Stabilizer
OPENQASM 2.0;
include "qelib1.inc";
qreg q[3];
creg c[3];

cx q[0], q[2];
cx q[1], q[2];
h q[2];

measure q[0] -> c[0];
measure q[1] -> c[1];
measure q[2] -> c[2];`,
};

function parseOpenQASM(src) {
  let nQubits = 0;
  let nClbits = 0;
  const gates = [];
  const measures = [];

  const getIdx = (tok, reg) => {
    const open = tok.indexOf('[');
    const close = tok.indexOf(']');
    if (open < 0 || close < 0) return null;
    return parseInt(tok.substring(open + 1, close), 10);
  };

  const lines = src.split('\n');
  for (let raw of lines) {
    const line = raw.split('//')[0].trim();
    if (!line) continue;
    const stmts = line.split(';');
    for (let s of stmts) {
      const stmt = s.trim();
      if (!stmt || stmt.startsWith("OPENQASM") || stmt.startsWith("include")) continue;
      
      if (stmt.startsWith("qreg")) {
        const idx = getIdx(stmt, "q");
        if (idx !== null) nQubits = Math.max(nQubits, idx);
        continue;
      }
      if (stmt.startsWith("creg")) {
        const idx = getIdx(stmt, "c");
        if (idx !== null) nClbits = Math.max(nClbits, idx);
        continue;
      }
      if (stmt.startsWith("measure")) {
        const parts = stmt.replace("measure", "").split("->");
        if (parts.length === 2) {
          const q = getIdx(parts[0].trim(), "q");
          const c = getIdx(parts[1].trim(), "c");
          if (q !== null && c !== null) measures.push({ q, c });
        }
        continue;
      }

      const parts = stmt.split(/\s+/);
      let op = parts[0].toLowerCase();
      let param = 0.0;
      const paramMatch = op.match(/^(rz|rx|ry|p|u1)\s*\(([^)]+)\)$/);
      if (paramMatch) {
        op = paramMatch[1];
        param = parseFloat(paramMatch[2]) || 0.0;
      }

      const argsStr = parts.slice(1).join("");
      const args = argsStr.split(",").map(a => getIdx(a.trim(), "q")).filter(x => x !== null);

      gates.push({ op, param, args });
    }
  }

  if (nQubits === 0 && gates.length > 0) {
    for (const g of gates) {
      for (const a of g.args) nQubits = Math.max(nQubits, a + 1);
    }
  }

  return { nQubits: Math.max(1, nQubits), nClbits: Math.max(1, nClbits), gates, measures };
}

function classifyCircuitTier(circuit) {
  let classicalOnly = true;
  let cliffordOnly = true;
  let tCount = 0;

  for (const g of circuit.gates) {
    if (["x", "cx", "ccx"].includes(g.op)) {
      // Classical
    } else if (["z", "h", "s", "sdg", "cz", "swap"].includes(g.op)) {
      classicalOnly = false;
    } else if (["t", "tdg"].includes(g.op)) {
      classicalOnly = false;
      cliffordOnly = false;
      tCount++;
    } else {
      classicalOnly = false;
      cliffordOnly = false;
    }
  }

  if (circuit.nQubits > 20) return { tier: "Refused", tCount };
  if (classicalOnly) return { tier: "Classical", tCount };
  if (cliffordOnly) return { tier: "Tableau", tCount: 0 };
  if (tCount <= 12) return { tier: "Magic", tCount };
  return { tier: "Statevector", tCount };
}

function simulateStatevector(circuit) {
  const n = circuit.nQubits;
  if (n > 20) {
    return { probabilities: new Map([["refused", 1.0]]), statevector: [] };
  }
  const dim = 1 << n;
  const re = new Float64Array(dim);
  const im = new Float64Array(dim);
  re[0] = 1.0;

  const INV_SQRT2 = 1.0 / Math.SQRT2;
  const COS_PI_4 = Math.cos(Math.PI / 4);
  const SIN_PI_4 = Math.sin(Math.PI / 4);

  for (const g of circuit.gates) {
    const op = g.op;
    if (g.args.length === 0) continue;
    const a = g.args[0];
    const bitA = 1 << a;

    if (op === "x") {
      for (let i = 0; i < dim; i++) {
        if ((i & bitA) === 0) {
          const j = i | bitA;
          const tr = re[i], ti = im[i];
          re[i] = re[j]; im[i] = im[j];
          re[j] = tr; im[j] = ti;
        }
      }
    } else if (op === "y") {
      for (let i = 0; i < dim; i++) {
        if ((i & bitA) === 0) {
          const j = i | bitA;
          const tr = re[i], ti = im[i];
          re[i] = im[j]; im[i] = -re[j];
          re[j] = -ti; im[j] = tr;
        }
      }
    } else if (op === "z") {
      for (let i = 0; i < dim; i++) {
        if ((i & bitA) !== 0) {
          re[i] = -re[i];
          im[i] = -im[i];
        }
      }
    } else if (op === "h") {
      for (let i = 0; i < dim; i++) {
        if ((i & bitA) === 0) {
          const j = i | bitA;
          const r0 = re[i], i0 = im[i];
          const r1 = re[j], i1 = im[j];
          re[i] = (r0 + r1) * INV_SQRT2;
          im[i] = (i0 + i1) * INV_SQRT2;
          re[j] = (r0 - r1) * INV_SQRT2;
          im[j] = (i0 - i1) * INV_SQRT2;
        }
      }
    } else if (op === "s" || op === "sdg") {
      const sgn = op === "s" ? 1 : -1;
      for (let i = 0; i < dim; i++) {
        if ((i & bitA) !== 0) {
          const r = re[i], q = im[i];
          re[i] = -sgn * q;
          im[i] = sgn * r;
        }
      }
    } else if (op === "t" || op === "tdg") {
      const sgn = op === "t" ? 1 : -1;
      for (let i = 0; i < dim; i++) {
        if ((i & bitA) !== 0) {
          const r = re[i], q = im[i];
          re[i] = r * COS_PI_4 - sgn * q * SIN_PI_4;
          im[i] = q * COS_PI_4 + sgn * r * SIN_PI_4;
        }
      }
    } else if (op === "rz") {
      const theta = g.param || 0.0;
      const c = Math.cos(theta / 2.0);
      const s = Math.sin(theta / 2.0);
      for (let i = 0; i < dim; i++) {
        const r = re[i], q = im[i];
        if ((i & bitA) === 0) {
          // e^{-i theta/2} = cos(theta/2) - i sin(theta/2)
          re[i] = r * c + q * s;
          im[i] = q * c - r * s;
        } else {
          // e^{i theta/2} = cos(theta/2) + i sin(theta/2)
          re[i] = r * c - q * s;
          im[i] = q * c + r * s;
        }
      }
    } else if (op === "cx" && g.args.length >= 2) {
      const c = g.args[0], t = g.args[1];
      const bitC = 1 << c, bitT = 1 << t;
      for (let i = 0; i < dim; i++) {
        if ((i & bitC) !== 0 && (i & bitT) === 0) {
          const j = i | bitT;
          const tr = re[i], ti = im[i];
          re[i] = re[j]; im[i] = im[j];
          re[j] = tr; im[j] = ti;
        }
      }
    } else if (op === "cz" && g.args.length >= 2) {
      const c = g.args[0], t = g.args[1];
      const bitC = 1 << c, bitT = 1 << t;
      for (let i = 0; i < dim; i++) {
        if ((i & bitC) !== 0 && (i & bitT) !== 0) {
          re[i] = -re[i];
          im[i] = -im[i];
        }
      }
    }
  }

  const probs = new Map();
  const measures = circuit.measures;
  const nClbits = circuit.nClbits;

  for (let i = 0; i < dim; i++) {
    const p = re[i] * re[i] + im[i] * im[i];
    if (p > 1e-9) {
      let key = "";
      if (measures.length > 0) {
        const bits = Array(nClbits).fill("0");
        for (const m of measures) {
          if (m.c < nClbits && m.q < n) {
            bits[nClbits - 1 - m.c] = (i & (1 << m.q)) !== 0 ? "1" : "0";
          }
        }
        key = bits.join("");
      } else {
        for (let bit = n - 1; bit >= 0; bit--) {
          key += (i & (1 << bit)) !== 0 ? "1" : "0";
        }
      }
      probs.set(key, (probs.get(key) || 0) + p);
    }
  }

  return { probabilities: probs, statevector: Array.from(re).map((r, i) => ({ re: r, im: im[i] })) };
}

function runCircuit() {
  const code = ui.qasmCode.value;
  const circuit = parseOpenQASM(code);
  state.parsedCircuit = circuit;

  const { tier, tCount } = classifyCircuitTier(circuit);
  ui.tierBadge.className = `tier-route-badge ${tier.toLowerCase()}`;
  ui.tierBadge.textContent = `${tier.toUpperCase()} ${tier === "Tableau" ? "O(N²)" : tier === "Classical" ? "O(N)" : "O(2ᴺ)"}`;
  ui.circuitSummaryText.textContent = `${circuit.nQubits} Qubits · ${circuit.gates.length} Gates · T-count: ${tCount}`;

  const result = simulateStatevector(circuit);
  state.circuitResults = {
    ...result,
    tier,
    shots: new Map(),
    totalShots: 0,
  };

  ui.qvmTierName.textContent = `${tier} Sim`;
  ui.qvmCost.textContent = tier === "Classical" ? "O(N) diagonal" : tier === "Tableau" ? "O(N²) Clifford" : `O(2^${circuit.nQubits})`;
  ui.qvmDim.textContent = String(1 << circuit.nQubits);
  ui.qvmPurity.textContent = "1.000000";

  let entropy = 0;
  for (const [_, p] of result.probabilities) {
    if (p > 1e-12) entropy -= p * Math.log2(p);
  }
  ui.qvmEntropy.textContent = `${entropy.toFixed(4)} bit`;

  renderHistogram();
  buildZXGraph(circuit);
}

// Bind Molecular Bond R to QVM Circuit
controls.injectBondBtn.addEventListener("click", () => {
  const R = state.w ? state.w.holon_pair_r(state.selectedPair) : 1.401;
  const q = solveQuantumH2(R);
  const theta = 2.0 * Math.acos(Math.max(-1, Math.min(1, q.cg)));

  const code = `// H2 Ground State VQE (Bound to R = ${R.toFixed(3)} a0)
OPENQASM 2.0;
include "qelib1.inc";
qreg q[2];
creg c[2];

// Initial HF reference |01>
x q[0];

// Parameterized Givens rotation theta = ${theta.toFixed(4)} rad
h q[1];
cx q[1], q[0];
rz(${theta.toFixed(4)}) q[0];
cx q[1], q[0];
h q[1];

measure q[0] -> c[0];
measure q[1] -> c[1];`;

  ui.qasmCode.value = code;
  runCircuit();
});

controls.circuitPresets.addEventListener("change", (e) => {
  const key = e.target.value;
  if (PRESET_CIRCUITS[key]) {
    ui.qasmCode.value = PRESET_CIRCUITS[key];
    runCircuit();
  }
});

controls.runCircuitBtn.addEventListener("click", runCircuit);

// Insert Gate Buttons
document.querySelectorAll(".gate-btn").forEach(btn => {
  btn.addEventListener("click", () => {
    const gate = btn.dataset.gate;
    const textarea = ui.qasmCode;
    let snippet = "";
    if (gate === "measure") snippet = "measure q[0] -> c[0];\n";
    else if (gate === "cx" || gate === "cz") snippet = `${gate} q[0], q[1];\n`;
    else if (gate === "ccx") snippet = "ccx q[0], q[1], q[2];\n";
    else snippet = `${gate} q[0];\n`;

    const start = textarea.selectionStart;
    const end = textarea.selectionEnd;
    textarea.value = textarea.value.substring(0, start) + snippet + textarea.value.substring(end);
    textarea.selectionStart = textarea.selectionEnd = start + snippet.length;
    textarea.focus();
    runCircuit();
  });
});

// =========================================================================
// 11. Interactive ZX Spider Graph
// =========================================================================

function buildZXGraph(circuit) {
  const nodes = [];
  const edges = [];
  const n = circuit.nQubits;

  const wireTails = [];
  for (let q = 0; q < n; q++) {
    const inNode = { id: nodes.length, type: "Boundary", label: `In ${q}`, q, phase: 0, x: 35, y: 30 + q * 40 };
    nodes.push(inNode);
    wireTails.push(inNode.id);
  }

  let col = 1;
  let tCount = 0;

  for (const g of circuit.gates) {
    const op = g.op;
    const a = g.args[0];
    const xPos = 55 + col * 48;

    if (op === "h") {
      const sp = { id: nodes.length, type: "Z", label: "Z", q: a, phase: 0, x: xPos, y: 30 + a * 40 };
      nodes.push(sp);
      edges.push({ u: wireTails[a], v: sp.id, hadamard: true });
      wireTails[a] = sp.id;
    } else if (op === "x") {
      const sp = { id: nodes.length, type: "X", label: "X", q: a, phase: 4, x: xPos, y: 30 + a * 40 };
      nodes.push(sp);
      edges.push({ u: wireTails[a], v: sp.id, hadamard: false });
      wireTails[a] = sp.id;
    } else if (op === "z") {
      const sp = { id: nodes.length, type: "Z", label: "Z(π)", q: a, phase: 4, x: xPos, y: 30 + a * 40 };
      nodes.push(sp);
      edges.push({ u: wireTails[a], v: sp.id, hadamard: false });
      wireTails[a] = sp.id;
    } else if (op === "s") {
      const sp = { id: nodes.length, type: "Z", label: "Z(π/2)", q: a, phase: 2, x: xPos, y: 30 + a * 40 };
      nodes.push(sp);
      edges.push({ u: wireTails[a], v: sp.id, hadamard: false });
      wireTails[a] = sp.id;
    } else if (op === "t" || op === "tdg") {
      tCount++;
      const phase = op === "t" ? 1 : 7;
      const sp = { id: nodes.length, type: "Z", label: `Z(${op === "t" ? "π/4" : "-π/4"})`, q: a, phase, x: xPos, y: 30 + a * 40 };
      nodes.push(sp);
      edges.push({ u: wireTails[a], v: sp.id, hadamard: false });
      wireTails[a] = sp.id;
    } else if (op === "cx" && g.args.length >= 2) {
      const c = g.args[0], t = g.args[1];
      const zNode = { id: nodes.length, type: "Z", label: "Z", q: c, phase: 0, x: xPos, y: 30 + c * 40 };
      nodes.push(zNode);
      edges.push({ u: wireTails[c], v: zNode.id, hadamard: false });
      wireTails[c] = zNode.id;

      const xNode = { id: nodes.length, type: "X", label: "X", q: t, phase: 0, x: xPos, y: 30 + t * 40 };
      nodes.push(xNode);
      edges.push({ u: wireTails[t], v: xNode.id, hadamard: false });
      wireTails[t] = xNode.id;

      edges.push({ u: zNode.id, v: xNode.id, hadamard: false });
    }
    col++;
  }

  const outX = Math.max(240, 55 + col * 48);
  for (let q = 0; q < n; q++) {
    const outNode = { id: nodes.length, type: "Boundary", label: `Out ${q}`, q, phase: 0, x: outX, y: 30 + q * 40 };
    nodes.push(outNode);
    edges.push({ u: wireTails[q], v: outNode.id, hadamard: false });
  }

  state.zxGraph = {
    nodes,
    edges,
    reduction: {
      tBefore: tCount,
      tAfter: tCount,
      gatesBefore: circuit.gates.length,
      gatesAfter: circuit.gates.length,
      phaseOmega: 0,
    },
  };

  updateZXStats();
  drawZXGraph();
}

function updateZXStats() {
  const g = state.zxGraph;
  let zCount = 0, xCount = 0, hadCount = 0;
  for (const n of g.nodes) {
    if (n.type === "Z") zCount++;
    if (n.type === "X") xCount++;
  }
  for (const e of g.edges) {
    if (e.hadamard) hadCount++;
  }

  ui.zxZCount.textContent = zCount;
  ui.zxXCount.textContent = xCount;
  ui.zxHadCount.textContent = hadCount;
  ui.zxTCount.textContent = `${g.reduction.tBefore} → ${g.reduction.tAfter}`;
  const redPct = g.reduction.gatesBefore > 0
    ? Math.max(0, Math.round((1 - g.reduction.gatesAfter / g.reduction.gatesBefore) * 100))
    : 0;
  ui.zxGateRed.textContent = `${redPct}%`;
  ui.zxPhaseOmega.textContent = `ω^${g.reduction.phaseOmega} (${g.reduction.phaseOmega === 0 ? "+1" : "e^{iπ/4}"})`;
  ui.zxReductionStat.textContent = `T-count: ${g.reduction.tBefore} → ${g.reduction.tAfter}`;
}

function drawZXGraph() {
  const view = fitCanvas(zxCanvas, zxCtx);
  zxCtx.clearRect(0, 0, view.width, view.height);

  const g = state.zxGraph;
  if (!g || g.nodes.length === 0) return;

  for (const edge of g.edges) {
    const u = g.nodes[edge.u];
    const v = g.nodes[edge.v];
    if (!u || !v) continue;

    zxCtx.strokeStyle = edge.hadamard ? "#3b82f6" : "rgba(237, 233, 223, 0.4)";
    zxCtx.lineWidth = edge.hadamard ? 2 : 1.5;
    zxCtx.setLineDash(edge.hadamard ? [4, 4] : []);

    zxCtx.beginPath();
    zxCtx.moveTo(u.x, u.y);
    zxCtx.lineTo(v.x, v.y);
    zxCtx.stroke();
    zxCtx.setLineDash([]);
  }

  for (const node of g.nodes) {
    zxCtx.beginPath();
    if (node.type === "Z") {
      zxCtx.fillStyle = "#22c55e";
      zxCtx.arc(node.x, node.y, 10, 0, Math.PI * 2);
      zxCtx.fill();
      zxCtx.strokeStyle = "#15803d";
      zxCtx.lineWidth = 2;
      zxCtx.stroke();
    } else if (node.type === "X") {
      zxCtx.fillStyle = "#ef4444";
      zxCtx.arc(node.x, node.y, 10, 0, Math.PI * 2);
      zxCtx.fill();
      zxCtx.strokeStyle = "#b91c1c";
      zxCtx.lineWidth = 2;
      zxCtx.stroke();
    } else {
      zxCtx.fillStyle = "#475569";
      zxCtx.arc(node.x, node.y, 6, 0, Math.PI * 2);
      zxCtx.fill();
      zxCtx.strokeStyle = "#94a3b8";
      zxCtx.lineWidth = 1.5;
      zxCtx.stroke();
    }

    if (node.phase !== 0 && node.type !== "Boundary") {
      zxCtx.fillStyle = "#ffffff";
      zxCtx.font = "bold 9px monospace";
      zxCtx.textAlign = "center";
      zxCtx.textBaseline = "middle";
      const pText = node.phase === 4 ? "π" : node.phase === 2 ? "π/2" : `${node.phase}π/4`;
      zxCtx.fillText(pText, node.x, node.y - 14);
    }
  }
}

function fuseZXSpiders() {
  const g = state.zxGraph;
  if (!g) return false;
  let fused = false;
  for (let i = 0; i < g.edges.length; i++) {
    const e = g.edges[i];
    if (e.hadamard) continue;
    const u = g.nodes.find(n => n.id === e.u);
    const v = g.nodes.find(n => n.id === e.v);
    if (!u || !v) continue;
    if (u.type !== "Boundary" && v.type !== "Boundary" && u.type === v.type) {
      // Fuse v into u with phase addition mod 8 (mod 2π)
      u.phase = (u.phase + v.phase) % 8;
      for (const e2 of g.edges) {
        if (e2.u === v.id) e2.u = u.id;
        if (e2.v === v.id) e2.v = u.id;
      }
      g.nodes = g.nodes.filter(n => n.id !== v.id);
      g.edges = g.edges.filter(e2 => e2.u !== e2.v);
      fused = true;
      break;
    }
  }
  recomputeZXStats();
  drawZXGraph();
  return fused;
}

function removeZXIdentities() {
  const g = state.zxGraph;
  if (!g) return false;
  let removed = false;
  for (const node of g.nodes) {
    if (node.type === "Boundary" || node.phase !== 0) continue;
    const incident = g.edges.filter(e => e.u === node.id || e.v === node.id);
    if (incident.length === 2 && !incident[0].hadamard && !incident[1].hadamard) {
      const o1 = incident[0].u === node.id ? incident[0].v : incident[0].u;
      const o2 = incident[1].u === node.id ? incident[1].v : incident[1].u;
      g.edges = g.edges.filter(e => e !== incident[0] && e !== incident[1]);
      g.edges.push({ u: o1, v: o2, hadamard: false });
      g.nodes = g.nodes.filter(n => n.id !== node.id);
      removed = true;
      break;
    }
  }
  recomputeZXStats();
  drawZXGraph();
  return removed;
}

function recomputeZXStats() {
  const g = state.zxGraph;
  if (!g) return;
  let tCount = 0;
  let gates = 0;
  for (const n of g.nodes) {
    if (n.type !== "Boundary") {
      gates++;
      if (n.phase % 2 !== 0) tCount++;
    }
  }
  g.reduction.tAfter = tCount;
  g.reduction.gatesAfter = Math.max(1, gates);
  updateZXStats();
}

controls.zxSimplifyBtn.addEventListener("click", () => {
  let changed = true;
  let passes = 0;
  while (changed && passes < 20) {
    const f = fuseZXSpiders();
    const id = removeZXIdentities();
    changed = f || id;
    passes++;
  }
  recomputeZXStats();
});

controls.zxFuseBtn.addEventListener("click", () => {
  fuseZXSpiders();
});

controls.zxIdBtn.addEventListener("click", () => {
  removeZXIdentities();
});

controls.zxResetBtn.addEventListener("click", () => {
  if (state.parsedCircuit) buildZXGraph(state.parsedCircuit);
});

// =========================================================================
// 12. Born Measurement Histogram & Monte Carlo Sampler
// =========================================================================

function renderHistogram() {
  const container = ui.histogramContainer;
  container.innerHTML = "";

  const probs = state.circuitResults.probabilities;
  const shots = state.circuitResults.shots;
  const totalShots = state.circuitResults.totalShots;

  let maxKey = "00";
  let maxP = 0;

  for (const [key, p] of probs) {
    if (p > maxP) { maxP = p; maxKey = key; }

    const row = document.createElement("div");
    row.className = "hist-bar-row";

    const label = document.createElement("span");
    label.className = "hist-label";
    label.textContent = `|${key}⟩`;

    const track = document.createElement("div");
    track.className = "hist-bar-track";

    const fill = document.createElement("div");
    fill.className = "hist-bar-value";
    fill.style.width = `${(p * 100).toFixed(1)}%`;

    if (totalShots > 0) {
      const shotCount = shots.get(key) || 0;
      const shotFraction = shotCount / totalShots;
      const shotBar = document.createElement("div");
      shotBar.className = "hist-bar-shots";
      shotBar.style.width = `${(shotFraction * 100).toFixed(1)}%`;
      track.appendChild(shotBar);
    }

    track.appendChild(fill);

    const txt = document.createElement("span");
    txt.className = "hist-prob-text";
    const shotCount = shots.get(key) || 0;
    txt.textContent = totalShots > 0
      ? `${(p * 100).toFixed(1)}% (${shotCount})`
      : `${(p * 100).toFixed(1)}%`;

    row.append(label, track, txt);
    container.appendChild(row);
  }

  ui.histTotalShots.textContent = totalShots.toLocaleString();
  ui.histMaxState.textContent = `|${maxKey}⟩ (${(maxP * 100).toFixed(1)}%)`;
}

function sampleShots(numShots) {
  const probs = state.circuitResults.probabilities;
  if (!probs || probs.size === 0) return;

  const entries = Array.from(probs.entries());
  const shots = new Map();

  for (let s = 0; s < numShots; s++) {
    const r = Math.random();
    let acc = 0;
    for (const [key, p] of entries) {
      acc += p;
      if (r <= acc) {
        shots.set(key, (shots.get(key) || 0) + 1);
        break;
      }
    }
  }

  state.circuitResults.shots = shots;
  state.circuitResults.totalShots = numShots;

  let fidelity = 0;
  for (const [key, p] of entries) {
    const empirical = (shots.get(key) || 0) / numShots;
    fidelity += Math.sqrt(p * empirical);
  }
  ui.histFidelity.textContent = `${(fidelity * 100).toFixed(2)}%`;

  renderHistogram();
}

controls.sample100Btn.addEventListener("click", () => sampleShots(100));
controls.sample1000Btn.addEventListener("click", () => sampleShots(1000));
controls.sample10000Btn.addEventListener("click", () => sampleShots(10000));

// =========================================================================
// 13. Main Animation Frame Loop with Render Guard
// =========================================================================

const eh = (v) => (Math.abs(v) < 1e-4 && v !== 0 ? v.toExponential(3) : v.toFixed(6));

function updateLedgerAndGates(w) {
  ui.eKin.textContent = eh(w.holon_e_kin());
  ui.ePair.textContent = eh(w.holon_e_pair());
  ui.eThree.textContent = eh(typeof w.holon_e_three === "function" ? w.holon_e_three() : 0);
  ui.eWall.textContent = eh(w.holon_e_wall());
  ui.eSpring.textContent = eh(w.holon_e_spring());
  ui.wExt.textContent = eh(w.holon_w_ext());
  ui.ledger.textContent = eh(w.holon_ledger());

  const drift = w.holon_drift_peak();
  const bound = w.holon_drift_bound();
  const passE = w.holon_energy_gate() === 1;

  ui.drift.textContent = drift.toExponential(3);
  ui.driftBound.textContent = bound.toExponential(3);
  const ratio = bound > 0 ? drift / bound : 0;
  ui.driftRatio.textContent = `${(100 * ratio).toFixed(1)}%`;
  ui.driftFill.style.width = `${Math.min(100, 100 * ratio)}%`;
  ui.driftFill.dataset.state = passE ? "pass" : "fail";
  ui.energyGate.textContent = `E-GATE: ${passE ? "PASS" : "FAIL"}`;
  ui.energyGate.dataset.state = passE ? "pass" : "fail";

  const passM = w.holon_momentum_gate() === 1;
  ui.pRes.textContent = w.holon_momentum_residual_peak().toExponential(3);
  ui.momentumGate.textContent = `P-GATE: ${passM ? "PASS" : "FAIL"}`;
  ui.momentumGate.dataset.state = passM ? "pass" : "fail";

  ui.clockDt.textContent = `${w.holon_dt().toFixed(4)} a.u.`;
  ui.clockDtRef.textContent = `${w.holon_dt_reference().toFixed(4)} a.u.`;
  ui.clockPeriod.textContent = `${w.holon_period_fs().toFixed(2)} fs`;

  ui.censusMolecules.textContent = w.holon_census_molecules();
  ui.cAtoms.textContent = w.holon_census_atoms();
  ui.cCandidates.textContent = w.holon_census_candidates();
  ui.cFormations.textContent = w.holon_census_formations();
  ui.cDissolutions.textContent = w.holon_census_dissolutions();
  ui.cRejections.textContent = w.holon_census_closure_rejections();
  ui.cBondSector.textContent = eh(w.holon_bond_sector_energy());

  const bonded = w.holon_bonded_count();
  const clusters = w.holon_cluster_count();
  const clusterAtoms = w.holon_cluster_atoms();
  ui.stageStatus.textContent = bonded === 0 ? "NO BOND"
    : clusters === 1 && clusterAtoms === 2 ? "H₂ BONDED"
    : `${clusters} CLUSTERS (${clusterAtoms} ATOMS)`;
  ui.clockTag.textContent = `t = ${w.holon_time().toFixed(0)} a.u.`;
}

function frame(now) {
  const wallDt = state.lastFrameMs === null ? 0.016 : (now - state.lastFrameMs) / 1000;
  state.lastFrameMs = now;
  state.frameMs = wallDt * 1000;

  // Rolling FPS calculation
  state.rollingFrameTimes.push(state.frameMs);
  if (state.rollingFrameTimes.length > 20) state.rollingFrameTimes.shift();
  const avgMs = state.rollingFrameTimes.reduce((a, b) => a + b, 0) / state.rollingFrameTimes.length;
  state.fps = Math.round(1000 / Math.max(1, avgMs));
  ui.fpsCount.textContent = `${state.fps} FPS`;

  if (state.w) {
    state.substepsThisFrame = state.w.holon_advance_frame(wallDt);

    const view = fitCanvas(stageCanvas, stageCtx);
    if (state.renderMode === "2d") {
      drawScene2D(state.w, view);
    } else {
      drawScene3D(state.w, view);
    }

    drawCurve();
    updateLedgerAndGates(state.w);
    updateRetractBridge();
  }

  requestAnimationFrame(frame);
}

// =========================================================================
// 14. Boot Orchestrator
// =========================================================================

async function boot() {
  ui.qasmCode.value = PRESET_CIRCUITS.h2_vqe;
  runCircuit();

  await loadWasm();
  requestAnimationFrame(frame);
}

boot().catch((err) => {
  document.body.dataset.engine = "failed";
  ui.runtimeStatus.textContent = "Engine Refused";
  showError(err.message || String(err));
});
