// Pixels and pointer input. Nothing else.
//
// Every holon, every materialization, every contact and every certificate lives in the
// Rust module. This file reads two flat f32 buffers out of its linear memory and draws
// them, and sends back three numbers when someone clicks.
//
// Two rules about linear memory, both learned the hard way.
//
// The buffer POINTERS are re-read every frame: publishing a frame can grow the
// underlying Vec, which moves it, and a cached pointer would then be drawing last
// frame's memory. Caching one is the classic way to make a wasm viewer show garbage
// that looks almost right.
//
// And `memory.buffer` is read AFTER the pointer, never before, because a call that
// grows wasm memory DETACHES the old ArrayBuffer — every view onto it becomes unusable,
// not stale. `new Float32Array(memory.buffer, ptr(), n)` evaluates `buffer` first and so
// would hand a detached one to the constructor if `ptr()` happened to allocate. It does
// not today; ordering it explicitly means it will not matter if that changes.

const canvas = document.querySelector("#stage");
const ctx = canvas.getContext("2d", { alpha: false });

const el = (id) => document.querySelector(`#${id}`);
const ui = {
  verdict: el("verdict"),
  verdictLabel: el("verdict").querySelector(".verdict-label"),
  verdictNote: el("verdict").querySelector(".verdict-note"),
  tier: el("tier"),
  tierName: el("tier-name"),
  speed: el("speed"),
  speedOut: el("speed-out"),
  grading: el("grading"),
  gradingOut: el("grading-out"),
  hint: el("hint"),
  title: el("tier-title"),
  plain: el("tier-plain"),
  g0: el("fact-g0"),
  terminal: el("fact-terminal"),
  domain: el("fact-domain"),
  ledger: el("fact-ledger"),
  evaluator: el("fact-evaluator"),
  certPlain: el("cert-plain"),
  holons: el("cert-holons"),
  mats: el("cert-mats"),
  ms: el("cert-ms"),
  required: el("cert-required"),
  impulse: el("cert-impulse"),
  disturb: el("cert-disturb"),
  cracked: el("cert-cracked"),
  honesty: el("honesty"),
  atoms: el("atoms"),
  unlock: el("unlock"),
  reroot: el("reroot"),
  strain: el("cert-strain"),
  strainPin: el("strain-pin"),
  scene: el("scene"),
  sceneOut: el("scene-out"),
  sceneControl: el("scene-control"),
  eps: el("cert-eps"),
  rem: el("cert-rem"),
  wfUnlock: el("wf-unlock"),
};

const NODE_STRIDE = 5;
const BOND_STRIDE = 5;

const state = {
  wasm: null,
  memory: null,
  tiers: [],
  scenes: [],
  weakField: [],
  last: performance.now(),
  thrown: false,
  // Wall clock for the throw event and for the solver's work budget. The engine has no
  // clock on wasm32-unknown-unknown — Instant panics there — so the side of the
  // boundary that HAS one owns the timing, and the engine keeps the physics.
  certifyMs: 0,
  frameMs: 0,
};

// How much of each frame the solver may have. The rest is for drawing and for the
// browser. 6 ms of a 16.7 ms frame leaves room on a mid-range laptop.
const SOLVER_TARGET_MS = 9;

const VERDICTS = {
  0: ["NOTHING CLAIMED YET", "Every cell in view is at least as fine as you can distinguish — that is the observer's claim, and it is served whether or not anything is happening. Nothing is moving, so no physics is claimed. Throw something and the engine certifies a frontier for that claim too."],
  1: ["CERTIFIED", "The resident frontier meets this tier's own resolution demand. The numbers below stand."],
  2: ["GRAIN FLOOR", "This tier cannot resolve what it is being asked, and no amount of extra resolution will change that — it is proved that refining inside a tier never makes a refused claim servable. Spending more holons here buys nothing. The only route to an answer is a different tier, and the certificate does not travel across that move: it would have to be earned again on the other side."],
  3: ["REFINEMENT UNAVAILABLE", "The frontier could not be refined far enough — the generator declined, or the declared holon budget for one throw was reached."],
  4: ["NO EVALUATOR", "Nothing here can be evaluated. The tier is real, its ledger is exact, and there is no validated way to run it."],
  5: ["NO GRAVITY CHART", "This scene has weight, and this engine has no certified way to make weight pull."],
  6: ["BUDGET EXHAUSTED", "The declared round budget for one throw ran out before a verdict was reached. That is an error, not a verdict."],
  8: ["OUTSIDE THE SCREEN", "This scene falls outside what its tier's weak-field chart can carry. Which way it falls, and what would lift it, is below — every one of these refusals names its own unlock, and one of them names why nothing ever will."],
  7: ["FLUX CEILING", "The loop is already carrying as much flux as it can. This is the only refusal here that is not about resolution: the state space has three values and there is no fourth, so the move has nowhere to go. Conjugate the charge or lower the loop, and it moves again."],
};

const REROOT = {
  1: (n) => `This tier's whole domain is <strong>exactly one</strong> terminal holon of the tier above it — one of those contains <strong>${count(n)}</strong> of these. The ledger identity across the zoom is exact and is checked to the last constituent.`,
  2: (n) => `This tier's domain spans <strong>${count(n)}</strong> whole terminal holons of the tier above. The zoom lands on a cell boundary, so the ledger follows.`,
  3: (n) => `This tier's domain sits <strong>inside</strong> one terminal holon of the tier above, filling ${(100 * n).toPrecision(3)}% of its width. A legitimate place to zoom to, and one that carries no exact count identity — said rather than smoothed over.`,
};

async function boot() {
  const response = await fetch("holon_sandbox.wasm");
  let result;
  try {
    result = await WebAssembly.instantiateStreaming(response.clone(), {});
  } catch {
    result = await WebAssembly.instantiate(await response.arrayBuffer(), {});
  }
  state.wasm = result.instance.exports;
  state.memory = state.wasm.memory;
  const text = readText();
  state.tiers = text.tiers;
  state.scenes = text.scenes;
  // The weak-field unlocks are the ENGINE's words, quoted rather than retyped here: a
  // refusal whose roadmap lives in the viewer is one that can silently diverge from it.
  state.weakField = text.weakField;

  ui.tier.max = String(state.tiers.length - 1);
  syncTier();
  syncSpeed();
  syncGrading();
  document.body.dataset.engine = "ready";
  requestAnimationFrame(frame);
}

// The tier table is built in Rust from the same values the physics reads, so the words
// on this page cannot drift from the engine behind it.
function readText() {
  const ptr = state.wasm.ciris_text_ptr();
  const len = state.wasm.ciris_text_len();
  const bytes = new Uint8Array(state.memory.buffer, ptr, len);
  return JSON.parse(new TextDecoder().decode(bytes));
}

function metres(value) {
  if (value === null || !Number.isFinite(value)) return "—";
  const units = [
    [1e-12, "pm"], [1e-9, "nm"], [1e-6, "µm"], [1e-3, "mm"],
    [1, "m"], [1e3, "km"],
  ];
  for (let i = units.length - 1; i >= 0; i -= 1) {
    const [scale, name] = units[i];
    if (Math.abs(value) >= scale) {
      const shown = value / scale;
      return `${shown < 10 ? shown.toFixed(2) : shown.toPrecision(4)} ${name}`;
    }
  }
  return `${value.toExponential(3)} m`;
}

function bigMetres(value) {
  if (value === null || !Number.isFinite(value)) return "—";
  if (value >= 9.4607e15) return `${(value / 9.4607e15).toPrecision(3)} light years`;
  if (value >= 1e3) return `${(value / 1e3).toPrecision(4)} km`;
  return metres(value);
}

function count(value) {
  const n = Number(value);
  if (!Number.isFinite(n)) return String(value);
  if (n < 1e6) return n.toLocaleString();
  return `${n.toExponential(3).replace("e+", " × 10^")}`;
}

function syncTier() {
  const index = Number(ui.tier.value);
  state.wasm.ciris_set_tier(index);
  state.thrown = false;
  const tier = state.tiers[index];
  ui.tierName.textContent = tier.name;
  ui.title.textContent = tier.name;
  ui.plain.textContent = tier.plain;
  ui.g0.textContent = tier.g0 === null ? "no length — not pinned" : metres(tier.g0);
  ui.terminal.textContent = tier.terminal;
  ui.domain.textContent = tier.domain === null ? "one plaquette" : bigMetres(tier.domain);
  ui.ledger.textContent = `${count(tier.constituents)} × ${tier.terminal}`;
  ui.evaluator.textContent = tier.evaluator;

  const over = tier.atoms.startsWith("over:");
  ui.atoms.dataset.over = over ? "1" : "0";
  if (tier.atoms === "n/a") {
    ui.atoms.dataset.over = "0";
    ui.atoms.textContent = "This tier has no length, so it has no atoms to count.";
  } else if (over) {
    const factor = Number(tier.atoms.slice(5));
    ui.atoms.textContent =
      `REFUSED — ${factor.toExponential(2).replace("e+", " × 10^")} times more than a ` +
      `64-bit count can hold. The ledger returns nothing rather than a wrong number.`;
  } else {
    ui.atoms.textContent = `${count(tier.atoms)} atoms — it fits, with room to spare.`;
  }

  if (tier.unlock) {
    ui.unlock.hidden = false;
    ui.unlock.textContent = tier.unlock;
  } else {
    ui.unlock.hidden = true;
  }

  const kind = state.wasm.ciris_reroot_kind();
  ui.reroot.dataset.kind = String(kind);
  ui.reroot.innerHTML = kind === 0
    ? "This is the outermost tier — there is nothing above it to re-root from."
    : REROOT[kind](state.wasm.ciris_reroot_ratio());

  ui.honesty.innerHTML = "";
  for (const line of honesty(tier, index)) {
    const item = document.createElement("li");
    item.innerHTML = line;
    ui.honesty.appendChild(item);
  }

  const scenes = state.scenes[index] ?? [];
  if (scenes.length > 1) {
    ui.sceneControl.hidden = false;
    ui.scene.max = String(scenes.length - 1);
    ui.scene.value = "0";
    state.wasm.ciris_set_gravity_scene(0);
    ui.sceneOut.textContent = scenes[0].name;
  } else {
    ui.sceneControl.hidden = true;
  }

  ui.hint.textContent = isGaugeTier()
    ? "Click to raise the loop's flux. C conjugates the charge, ↓ lowers it."
    : "Click or tap anywhere in the box to throw.";
  resetCertificate(tier);
}

// What this tier is knowingly not. Assembled from the tier's own values, so a tier
// cannot quietly stop disclosing something.
function honesty(tier, index) {
  const lines = [];
  lines.push(
    "The picture is <strong>two-dimensional</strong>; the ledger is three. Each cell " +
    "you see stands for a column of real matter, and its mass is that column's."
  );
  if (tier.refusal) {
    lines.push(`<strong>Refuses:</strong> ${tier.refusal}`);
  }
  if (tier.lch !== null && tier.required !== null) {
    const shortfall = tier.required < tier.g0 ? tier.g0 / tier.required : 0;
    lines.push(
      `Its process zone is <strong>${metres(tier.lch)}</strong>, so a crack claim here ` +
      `needs cells of ${metres(tier.required)}` +
      (shortfall > 1
        ? ` — <strong>${shortfall.toPrecision(3)}× finer than anything that exists at this tier</strong>, ` +
          "which is why it refuses however many holons you spend on it."
        : ".")
    );
  }
  if (index === 0) {
    lines.push(
      "This tier is <strong>not an approximation of anything</strong>. Four links, three " +
      "flux values each, a Gauss law that is a machine predicate over them, and a ground " +
      "state from a 3×3 eigenproblem. The error bound is zero because there is no coarse " +
      "view to be honest about."
    );
    lines.push(
      "Link charge conjugation acts on the taxonomy's route object as <strong>time " +
      "reversal</strong> — an exact machine-checked dictionary entry. It is <strong>not" +
      "</strong> a shared carrier: the route→gauge identification was killed by machine " +
      "(Core/RouteGauge.lean). One finite symmetry read in two languages, and nothing more."
    );
  }
  if (tier.evaluator === "geodesic, weak-field chart") {
    lines.push(
      "Weight pulls here <strong>by certificate</strong>, not because gravity was " +
      "switched on. The chart is a declared weak-field one, the scene declares an " +
      "envelope, and a screen measures ε over it — what you are watching is licensed " +
      "to the accuracy shown, and nothing further."
    );
    lines.push(
      "No mass appears anywhere in the motion. Universality of free fall is a property " +
      "of how the geodesic is constructed, not a rule imposed on top of it."
    );
    lines.push(
      "What you see is <strong>one body on a declared chart. No matter field is " +
      "materialized at this tier</strong> — the ledger above is counted, not resolved. " +
      "Refusing to draw matter coarser than you can see is a different thing from not " +
      "drawing it at all, and this tier is the second."
    );
    lines.push(
      "The path itself IS drawn at your acuity: it is sampled once per integration " +
      "step, closer together than you can distinguish, because the straight line " +
      "between two computed points is interpolation and would otherwise be visible."
    );
  }
  if (index === 5) {
    lines.push(
      "The same arithmetic is why a GPS satellite clock runs about <strong>45.7 " +
      "microseconds a day</strong> fast — the gravitational part of it. That is a " +
      "cross-check on the size of the effect, not something this demo measures."
    );
  }
  if (index === 3) {
    lines.push(
      "Sand does not crack, it <strong>pours</strong> — and the engine works that out " +
      "rather than being told. Quartz's own numbers refuse a cohesive law at a " +
      "half-millimetre cell by a factor of a hundred, and contact is what is left."
    );
    lines.push(
      "The contact is <strong>softer than quartz</strong>, by a declared criterion: the " +
      "least stiffness whose overlap stays under 5% of a grain. Real quartz stiffness " +
      "cannot be stepped explicitly at interactive rates, here or anywhere."
    );
  }
  if (index === 4) {
    lines.push(
      "Fracture energy here is <strong>110 J/m²</strong>; the crystal tier exports about " +
      "<strong>1</strong>. The hundredfold difference is minted at this scale, not " +
      "carried up from below — so the two numbers are different kinds of thing and the " +
      "zoom never interpolates between them."
    );
  }
  if (tier.evaluator !== "none" && index !== 0) {
    lines.push(
      "The scene is <strong>never coarser than you can see</strong>. Your acuity is a " +
      "claim on the frontier exactly like the physics is, so the certified grain is the " +
      "join of the two: at least this fine everywhere in view, and finer than this " +
      "wherever the impact needs it."
    );
    lines.push(
      "Most of those cells are <strong>resident but not moving</strong>. Sand that has " +
      "been sitting in a box is sitting in a box; only what a throw disturbs is stepped, " +
      "which is what makes a hundred thousand cells affordable at all. A still cell is " +
      "treated as fixed while it rests, and that costs the reported impulse about " +
      "<strong>1%</strong> — measured against a control with resting switched off, and " +
      "printed on the number rather than left implicit."
    );
  }
  lines.push(
    "Gravity is <strong>chart data</strong>, one uniform value for the whole scene. " +
    "Nothing here has its own gravity, and nothing pulls on anything else."
  );
  return lines;
}

function isGaugeTier() {
  return Number(ui.tier.value) === 0;
}

function resetCertificate(tier) {
  const [label, note] = VERDICTS[0];
  ui.verdict.dataset.code = "0";
  ui.verdictLabel.textContent = label;
  ui.verdictNote.textContent = tier.evaluator === "none"
    ? "This tier will refuse. Throw at it anyway — the refusal is the point."
    : note;
  ui.certPlain.textContent =
    "Nothing has been thrown at this tier yet. The cells you see are pinned at your own " +
    "resolving power — the frame includes the observer.";
  for (const node of [ui.holons, ui.mats, ui.ms, ui.required, ui.impulse, ui.disturb, ui.cracked]) {
    node.textContent = "—";
  }
}

function syncSpeed() {
  ui.speedOut.textContent = Number(ui.speed.value).toFixed(2);
}

// The slider is logarithmic: the demand is worth changing by factors, not by steps.
function gradingValue() {
  return Math.pow(2, Number(ui.grading.value));
}

function syncGrading() {
  const grading = gradingValue();
  state.wasm.ciris_set_grading(grading);
  ui.gradingOut.textContent = grading.toFixed(3);
}

function throwAt(event) {
  const rect = canvas.getBoundingClientRect();
  const x = (event.clientX - rect.left) / rect.width;
  const y = 1 - (event.clientY - rect.top) / rect.height;
  const started = performance.now();
  state.wasm.ciris_throw(x, y, Number(ui.speed.value));
  state.certifyMs = performance.now() - started;
  state.thrown = true;
  readCertificate();
}

function readCertificate() {
  const w = state.wasm;
  const code = w.ciris_verdict();
  const [label, note] = VERDICTS[code] ?? VERDICTS[0];
  ui.verdict.dataset.code = String(code);
  ui.verdictLabel.textContent = label;
  ui.verdictNote.textContent = note;

  const tier = state.tiers[Number(ui.tier.value)];
  ui.holons.textContent =
    `${w.ciris_holons().toLocaleString()} resident, ` +
    `${w.ciris_node_count().toLocaleString()} carrying matter, ` +
    `${w.ciris_awake_count().toLocaleString()} moving`;
  ui.mats.textContent = `${w.ciris_materializations().toLocaleString()} in ${w.ciris_rounds().toLocaleString()} rounds`;
  ui.ms.textContent = `${state.certifyMs.toFixed(2)} ms`;
  ui.required.textContent =
    `${metres(w.ciris_acuity())} (you)` +
    (tier.required === null ? "" : `, ${metres(tier.required)} (physics)`);
  ui.cracked.textContent = w.ciris_cracked().toLocaleString();

  // The strain-rate pin. Both tiers that certify here land in the band no experiment
  // covers, and the panel says so where the number is rather than in a footnote.
  const strain = w.ciris_strain_rate();
  if (strain > 0) {
    ui.strain.textContent = `${strain.toExponential(2).replace("e+", " × 10^")} s⁻¹`;
    const low = w.ciris_strain_gap_low();
    const high = w.ciris_strain_gap_high();
    if (strain > low && strain < high) {
      ui.strainPin.hidden = false;
      ui.strainPin.innerHTML =
        "This throw lands in the strain-rate band <strong>no experiment covers</strong>. " +
        "Molecular dynamics certifies above 10^7 per second, laboratory tests below 1, " +
        "and between them is twelve decades no simulation crosses. The material numbers " +
        "here reach this rate by interpolation, and that is a pin you are accepting, not " +
        "a result.";
    } else {
      ui.strainPin.hidden = true;
    }
  } else {
    ui.strain.textContent = "—";
    ui.strainPin.hidden = true;
  }

  // The weak-field screen, where the tier has one.
  const eps = w.ciris_weak_field_epsilon();
  if (eps > 0) {
    const stake = w.ciris_weak_field_stake();
    ui.eps.textContent =
      `${eps.toExponential(3).replace("e-", " × 10⁻")} (stake ${stake.toExponential(0).replace("e-", "e−")})`;
    const rem = w.ciris_weak_field_remainder();
    ui.rem.textContent = rem > 0
      ? rem.toExponential(3).replace("e-", " × 10⁻") + " per dynamical time"
      : "unbounded — the scene is outside the screen";
    const code = w.ciris_weak_field_refusal();
    if (code > 0) {
      const entry = state.weakField[code] ?? { unlock: "", ceiling: false };
      ui.wfUnlock.hidden = false;
      ui.wfUnlock.innerHTML =
        `<strong>${entry.ceiling ? "Ceiling" : "Floor"} — ${entry.name}.</strong> ${entry.unlock}`;
    } else {
      ui.wfUnlock.hidden = true;
    }
  } else {
    ui.eps.textContent = "—";
    ui.rem.textContent = "—";
    ui.wfUnlock.hidden = true;
  }

  const refusal = w.ciris_law_refusal();
  if (w.ciris_gravity_scene_count() > 0) {
    const scenes = state.scenes[Number(ui.tier.value)] ?? [];
    const scene = scenes[w.ciris_gravity_scene()] ?? { plain: "" };
    if (code === 8) {
      ui.certPlain.textContent =
        `${scene.plain} The screen refuses it, and the refusal below says what would ` +
        `carry it instead.`;
    } else if (code === 2) {
      ui.certPlain.textContent =
        `${scene.plain} Curvature and flatness are indistinguishable at this size in ` +
        `64-bit arithmetic, so the FLAT chart is the licensed answer — not a failure, ` +
        `a seam.`;
    } else {
      ui.certPlain.textContent = `${scene.plain} Weight pulls here by certificate.`;
    }
  } else if (code === 4 || code === 5) {
    ui.certPlain.textContent =
      "No frontier was built. Refinement cannot rescue a claim that has no way to be " +
      "evaluated, so nothing was materialized at all.";
  } else if (refusal === 1) {
    ui.certPlain.textContent =
      "The relations here carry no cohesive law: at this frontier's spacing the " +
      "homogenizer refuses one. These grains are in contact, not bonded.";
  } else {
    ui.certPlain.textContent =
      `Stepping at ${(1e6 * w.ciris_dt()).toPrecision(3)} µs per step, ` +
      `${(100 * w.ciris_slow_motion()).toPrecision(3)}% of real time. ` +
      (w.ciris_softening() > 1.5
        ? `Contact softened ${w.ciris_softening().toPrecision(3)}× against the material named above.`
        : "Contact at the stiffness this tier's own material implies.");
  }
}

function frame(now) {
  const elapsed = Math.min((now - state.last) / 1000, 1 / 20);
  state.last = now;
  if (state.thrown || state.wasm.ciris_gravity_scene_count() > 0) {
    const started = performance.now();
    state.wasm.ciris_step(elapsed);
    state.frameMs = performance.now() - started;
    tuneWorkBudget();
    ui.impulse.textContent =
    `${state.wasm.ciris_net_impulse().toPrecision(3)} N·s ±1%` +
    (state.wasm.ciris_impulse() > state.wasm.ciris_net_impulse() * 1.05
      ? ` (${(state.wasm.ciris_impulse() / state.wasm.ciris_net_impulse()).toPrecision(3)}× that in total pushing)`
      : "");
    ui.disturb.textContent = metres(state.wasm.ciris_disturbance());
  }
  draw();
  requestAnimationFrame(frame);
}

// Move the solver's work budget toward the frame share it is allowed. A constant
// cannot do this job: the same scene measured 8 ms natively and 30 ms through wasm,
// and a reader's laptop is a third number again. The loop is deliberately slow — a
// quarter of the way each frame, clamped to a factor of two — so it settles rather
// than oscillating, and the consequence of whatever it settles on is shown honestly as
// the slow-motion factor.
function tuneWorkBudget() {
  if (state.frameMs <= 0) return;
  const budget = state.wasm.ciris_work_budget();
  const wanted = budget * (SOLVER_TARGET_MS / state.frameMs);
  const clamped = Math.min(Math.max(wanted, budget * 0.5), budget * 2);
  const next = Math.round(budget + (clamped - budget) * 0.25);
  if (Math.abs(next - budget) > budget * 0.02) {
    state.wasm.ciris_set_work_budget(Math.max(64, Math.min(next, 4000000)));
  }
}

// Two layers. The STILL layer is every resident cell that is not moving, drawn once
// into an offscreen canvas and blitted each frame; the awake cells are drawn over it.
//
// This is a caching of what is resident, not a substitute for it. Each dot on the still
// layer is a holon the engine is carrying and the certificate covers — the layer is
// rebuilt from those holons whenever the set of still ones changes. It is emphatically
// NOT an interpolation over a coarse frontier: at 118,296 cells the alternative is
// 118,296 draw calls a frame, and the scene is that fine because the observer's claim
// says it must be.
const still = document.createElement("canvas");
const stillCtx = still.getContext("2d", { alpha: true });
let stillKey = null;

function drawStillLayer(scale, size) {
  const w = state.wasm;
  const count = w.ciris_node_count();
  still.width = size;
  still.height = size;
  stillCtx.clearRect(0, 0, size, size);
  if (count === 0) return;
  const nodePtr = w.ciris_node_ptr();
  const nodes = new Float32Array(state.memory.buffer, nodePtr, count * NODE_STRIDE);
  const tier = state.tiers[Number(ui.tier.value)];
  const cohesive = tier.evaluator === "cohesive relations";
  stillCtx.fillStyle = cohesive ? "#5d666d" : "#b98c33";
  for (let i = 0; i < count; i += 1) {
    if (nodes[i * NODE_STRIDE + 4] > 0) continue; // moving: the live layer draws it
    const r = nodes[i * NODE_STRIDE + 2] * scale;
    const x = nodes[i * NODE_STRIDE] * scale - r;
    const y = size - nodes[i * NODE_STRIDE + 1] * scale - r;
    // A cell is a square region of the chart, so a square is what it is. It is also
    // what a canvas can draw a hundred thousand of.
    stillCtx.fillRect(x, y, Math.max(2 * r, 1), Math.max(2 * r, 1));
  }
}

function draw() {
  const w = state.wasm;
  const domain = w.ciris_domain();
  const size = canvas.width;
  const scale = Number.isFinite(domain) && domain > 0 ? size / domain : 1;
  const toX = (x) => x * scale;
  const toY = (y) => size - y * scale;

  ctx.fillStyle = "#0a0908";
  ctx.fillRect(0, 0, size, size);

  const tier = state.tiers[Number(ui.tier.value)];
  if (!Number.isFinite(domain)) {
    drawPlaquette(size);
    return;
  }

  if (w.ciris_gravity_scene_count() > 0) {
    drawGravityScene(tier, size);
    return;
  }

  const count = w.ciris_node_count();
  if (count === 0) {
    drawLedgerOnly(tier, size);
    return;
  }

  // Rebuild the still layer only when the set of still cells can have changed.
  const key = `${ui.tier.value}|${count}|${w.ciris_sleep_generation()}|${w.ciris_awake_count()}`;
  if (key !== stillKey) {
    stillKey = key;
    drawStillLayer(scale, size);
  }
  ctx.drawImage(still, 0, 0);

  const nodePtr = w.ciris_node_ptr();
  const nodes = new Float32Array(state.memory.buffer, nodePtr, count * NODE_STRIDE);
  const cohesive = tier.evaluator === "cohesive relations";
  for (let i = 0; i < count; i += 1) {
    const speed = nodes[i * NODE_STRIDE + 4];
    if (speed <= 0) continue;
    const r = nodes[i * NODE_STRIDE + 2] * scale;
    const x = nodes[i * NODE_STRIDE] * scale;
    const y = size - nodes[i * NODE_STRIDE + 1] * scale;
    const heat = Math.min(speed / 2.5, 1);
    ctx.fillStyle = cohesive
      ? mix("#5d666d", "#e8c07a", heat)
      : mix("#b98c33", "#f6dfa4", heat);
    ctx.fillRect(x - r, y - r, Math.max(2 * r, 1.5), Math.max(2 * r, 1.5));
  }

  const bonds = w.ciris_bond_count();
  if (bonds > 0) {
    const bondPtr = w.ciris_bond_ptr();
    const buffer = new Float32Array(state.memory.buffer, bondPtr, bonds * BOND_STRIDE);
    ctx.lineWidth = 1.4;
    for (let i = 0; i < bonds; i += 1) {
      const damage = buffer[i * BOND_STRIDE + 4];
      // A crack is not drawn as a crack. It is the set of relation holons at full
      // damage, and that is literally what is on screen.
      if (damage <= 0.01) continue;
      ctx.strokeStyle = damage >= 1 ? "#e0553b" : `rgba(224,85,59,${0.15 + 0.7 * damage})`;
      ctx.beginPath();
      ctx.moveTo(toX(buffer[i * BOND_STRIDE]), toY(buffer[i * BOND_STRIDE + 1]));
      ctx.lineTo(toX(buffer[i * BOND_STRIDE + 2]), toY(buffer[i * BOND_STRIDE + 3]));
      ctx.stroke();
    }
  }

  if (w.ciris_projectile_live()) {
    const px = toX(w.ciris_projectile_x());
    const py = toY(w.ciris_projectile_y());
    const pr = Math.max(w.ciris_projectile_r() * scale, 2);
    ctx.beginPath();
    ctx.arc(px, py, pr, 0, Math.PI * 2);
    ctx.fillStyle = "#6fa8dc";
    ctx.fill();
  }
}

// The vacuum tier has no metres, so it gets no metre grid: four oriented links, the
// flux they carry, and the ground state they sit in. All of it read out of the engine —
// the arrows show the flux the plaquette actually holds, and the bars are the ground
// state of a 3x3 eigenproblem the core solves, not a drawing.
function drawPlaquette(size) {
  const w = state.wasm;
  const flux = w.ciris_gauge_flux();
  const closed = w.ciris_gauge_closed();
  const m = size * 0.24;
  const a = size * 0.46 - m;
  const b = size * 0.46 + m;

  const hue = flux > 0 ? "#6fa8dc" : flux < 0 ? "#e2a13c" : "#7e8a93";
  ctx.strokeStyle = hue;
  ctx.lineWidth = 3 + 2 * Math.abs(flux);
  ctx.strokeRect(a, a, 2 * m, 2 * m);

  // One arrowhead per link, pointing the way the flux runs.
  if (flux !== 0) {
    const dir = flux > 0 ? 1 : -1;
    const mid = (p, q) => (p + q) / 2;
    const heads = [
      [mid(a, b), a, dir, 0], [b, mid(a, b), 0, dir],
      [mid(a, b), b, -dir, 0], [a, mid(a, b), 0, -dir],
    ];
    ctx.fillStyle = hue;
    for (const [x, y, dx, dy] of heads) {
      ctx.beginPath();
      ctx.moveTo(x + dx * 13, y + dy * 13);
      ctx.lineTo(x - dy * 8 - dx * 6, y + dx * 8 - dy * 6);
      ctx.lineTo(x + dy * 8 - dx * 6, y - dx * 8 - dy * 6);
      ctx.closePath();
      ctx.fill();
    }
  }

  ctx.fillStyle = "#e8e4dc";
  for (const [x, y] of [[a, a], [b, a], [b, b], [a, b]]) {
    ctx.beginPath();
    ctx.arc(x, y, 9, 0, Math.PI * 2);
    ctx.fill();
  }

  ctx.textAlign = "center";
  ctx.fillStyle = "#e8e4dc";
  ctx.font = "600 22px ui-sans-serif, sans-serif";
  ctx.fillText(`loop flux ${flux > 0 ? "+" : ""}${flux}`, size / 2, size * 0.46);
  ctx.font = "400 15px ui-sans-serif, sans-serif";
  ctx.fillStyle = closed ? "#6cc06c" : "#c8564a";
  ctx.fillText(
    closed ? "Gauss law holds at all four corners" : "GAUSS LAW BROKEN",
    size / 2,
    size * 0.46 + 26,
  );

  // The ground state, as the engine computes it.
  ctx.fillStyle = "#9a958b";
  ctx.font = "400 14px ui-sans-serif, sans-serif";
  ctx.textAlign = "left";
  ctx.fillText("vacuum, by flux sector", size * 0.20, b + 76);
  const labels = ["−1", "0", "+1"];
  for (let i = 0; i < 3; i += 1) {
    const p = w.ciris_gauge_vacuum(i);
    const y = b + 96 + i * 26;
    ctx.fillStyle = i - 1 === flux ? "#d9a441" : "#6a655c";
    ctx.fillText(labels[i], size * 0.20, y + 10);
    ctx.fillStyle = "#2c2823";
    ctx.fillRect(size * 0.28, y, size * 0.40, 12);
    ctx.fillStyle = i - 1 === flux ? "#d9a441" : "#6fa8dc";
    ctx.fillRect(size * 0.28, y, size * 0.40 * p, 12);
    ctx.fillStyle = "#9a958b";
    ctx.fillText(p.toFixed(4), size * 0.70, y + 10);
  }
  ctx.textAlign = "center";
}

// A gravity tier draws its SCENE, at the scene's own extent.
//
// That extent is not the tier's domain, and the difference is the point: a hundred
// metres of thrown ball inside a 12,742 km planet, one star's orbit inside a hundred
// thousand light years. Drawing the claim at the ledger's extent would render every one
// of these as a single pixel.
function drawGravityScene(tier, size) {
  const w = state.wasm;
  const view = w.ciris_gravity_view_m();
  if (!(view > 0)) return;
  const scale = size / view;
  const cx = size / 2;
  const cy = size / 2;
  const toX = (x) => cx + x * scale;
  const toY = (y) => cy - y * scale;

  const refused = w.ciris_weak_field_refusal() > 0;

  // The declared patch or envelope, as the region the claim is ABOUT.
  ctx.strokeStyle = refused ? "rgba(200,86,74,.55)" : "rgba(111,168,220,.35)";
  ctx.setLineDash([6, 6]);
  ctx.lineWidth = 1.5;
  ctx.strokeRect(size * 0.08, size * 0.08, size * 0.84, size * 0.84);
  ctx.setLineDash([]);

  // A central mass, where the scene has one.
  const trailCount = w.ciris_trail_count();
  if (trailCount > 0 && tier.name !== "planet") {
    ctx.beginPath();
    ctx.arc(cx, cy, 7, 0, Math.PI * 2);
    ctx.fillStyle = "#e8e4dc";
    ctx.fill();
  }

  if (trailCount > 1) {
    const ptr = w.ciris_trail_ptr();
    const trail = new Float64Array(state.memory.buffer, ptr, trailCount * 2);
    ctx.strokeStyle = "#6fa8dc";
    ctx.lineWidth = 2;
    ctx.beginPath();
    for (let i = 0; i < trailCount; i += 1) {
      const x = toX(trail[i * 2]);
      const y = toY(trail[i * 2 + 1]);
      if (i === 0) ctx.moveTo(x, y); else ctx.lineTo(x, y);
    }
    ctx.stroke();
    // The body itself, at the head of its own trail.
    ctx.beginPath();
    ctx.arc(toX(trail[(trailCount - 1) * 2]), toY(trail[(trailCount - 1) * 2 + 1]), 6, 0, Math.PI * 2);
    ctx.fillStyle = w.ciris_landed() ? "#6cc06c" : "#f6dfa4";
    ctx.fill();
  }

  ctx.textAlign = "center";
  ctx.fillStyle = "#9a958b";
  ctx.font = "400 15px ui-sans-serif, sans-serif";
  const scenes = state.scenes[Number(ui.tier.value)] ?? [];
  const scene = scenes[w.ciris_gravity_scene()];
  if (scene) ctx.fillText(scene.name, size / 2, size - 46);
  ctx.fillStyle = "#6a655c";
  ctx.font = "400 13px ui-sans-serif, sans-serif";
  ctx.fillText(`view ${bigMetres(view)} across`, size / 2, size - 24);
  if (trailCount === 0) {
    ctx.fillStyle = refused ? "#c8564a" : "#6a655c";
    ctx.font = "400 15px ui-sans-serif, sans-serif";
    ctx.fillText(
      refused ? "nothing is drawn: the screen refused this scene" : "nothing moves here — the claim is the certificate",
      size / 2,
      size / 2,
    );
  }
}

// A tier with a ledger and no dynamics gets its ledger drawn, and nothing pretending to
// move.
function drawLedgerOnly(tier, size) {
  ctx.fillStyle = "#1a1815";
  ctx.fillRect(0, size * (1 - (tier.name === "crystal" ? 1 : 0.6)), size, size);
  ctx.fillStyle = "#9a958b";
  ctx.textAlign = "center";
  ctx.font = "500 19px ui-sans-serif, sans-serif";
  ctx.fillText(`${count(tier.constituents)} × ${tier.terminal}`, size / 2, size / 2 - 12);
  ctx.font = "400 15px ui-sans-serif, sans-serif";
  ctx.fillStyle = "#6a655c";
  ctx.fillText("exact ledger, no certified dynamics", size / 2, size / 2 + 16);
}

function mix(from, to, t) {
  const parse = (hex) => [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
  const [ar, ag, ab] = parse(from);
  const [br, bg, bb] = parse(to);
  const c = (a, b) => Math.round(a + (b - a) * t);
  return `rgb(${c(ar, br)},${c(ag, bg)},${c(ab, bb)})`;
}

canvas.addEventListener("pointerdown", throwAt);
ui.tier.addEventListener("input", syncTier);
ui.scene.addEventListener("input", () => {
  const index = Number(ui.scene.value);
  state.wasm.ciris_set_gravity_scene(index);
  const scenes = state.scenes[Number(ui.tier.value)] ?? [];
  ui.sceneOut.textContent = scenes[index]?.name ?? "—";
  state.thrown = true;
  readCertificate();
});
window.addEventListener("keydown", (event) => {
  if (!isGaugeTier()) return;
  if (event.key === "c" || event.key === "C") {
    state.wasm.ciris_gauge_conjugate();
    readCertificate();
  } else if (event.key === "ArrowDown") {
    state.wasm.ciris_gauge_lower();
    readCertificate();
  }
});
ui.speed.addEventListener("input", syncSpeed);
ui.grading.addEventListener("input", () => {
  syncGrading();
  ui.hint.textContent = "Throw again to see the demand take effect — the frontier is certified per throw.";
});

boot().catch((error) => {
  document.body.dataset.engine = "failed";
  ui.verdictLabel.textContent = "ENGINE FAILED TO LOAD";
  ui.verdictNote.textContent = String(error);
});
