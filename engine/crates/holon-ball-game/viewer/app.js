const canvas = document.querySelector("#stage");
const ctx = canvas.getContext("2d");
const resetButton = document.querySelector("#reset-button");
const speedControl = document.querySelector("#speed-control");
const speedOutput = document.querySelector("#speed-output");

const ui = Object.fromEntries(
  [
    "runtime-status", "stage-status", "clock", "aim-copy", "resident-count",
    "crack-count", "damage-fill", "max-damage", "density", "young", "tensile",
    "fracture", "ball-speed", "contact-force", "impact-count",
  ].map((id) => [id, document.querySelector(`#${id}`)]),
);

const WORLD = { width: 12.1, height: 6.25 };
const WALL = { x: 7.05, bottom: 1.28, top: 4.955 };
const BALL_RADIUS = 0.34;
const NODE_RADIUS = 0.098;
const state = {
  wasm: null,
  nodes: [],
  bonds: [],
  running: false,
  launched: false,
  pointerY: (WALL.bottom + WALL.top) * 0.5,
  lastFrame: performance.now(),
};

async function loadWasm() {
  const response = await fetch("holon_ball_game.wasm");
  let result;
  try {
    result = await WebAssembly.instantiateStreaming(response.clone(), {});
  } catch {
    result = await WebAssembly.instantiate(await response.arrayBuffer(), {});
  }
  state.wasm = result.instance.exports;
  state.wasm.ciris_reset();
  readTopology();
  readMaterial();
  document.body.dataset.engine = "ready";
  ui["runtime-status"].textContent = "Rust/WASM dynamics live";
  ui["stage-status"].textContent = "READY TO THROW";
}

function readTopology() {
  const w = state.wasm;
  const nodeCount = w.ciris_node_count();
  const bondCount = w.ciris_bond_count();
  state.nodes = Array.from({ length: nodeCount }, () => ({ x: 0, y: 0, anchored: false }));
  state.bonds = Array.from({ length: bondCount }, (_, index) => ({
    a: w.ciris_bond_a(index),
    b: w.ciris_bond_b(index),
    weak: Boolean(w.ciris_bond_is_weak(index)),
    damage: 0,
  }));
  ui["resident-count"].textContent = nodeCount.toLocaleString();
}

function readMaterial() {
  const w = state.wasm;
  ui.density.textContent = `${w.ciris_material_density().toLocaleString()} kg/m³`;
  ui.young.textContent = `${(w.ciris_material_young_modulus() / 1e9).toFixed(0)} GPa`;
  ui.tensile.textContent = `${(w.ciris_material_tensile_strength() / 1e6).toFixed(1)} MPa`;
  ui.fracture.textContent = `${w.ciris_material_fracture_energy().toFixed(0)} J/m²`;
}

function fitCanvas() {
  const ratio = Math.min(window.devicePixelRatio || 1, 2);
  const rect = canvas.getBoundingClientRect();
  const width = Math.max(1, Math.round(rect.width * ratio));
  const height = Math.max(1, Math.round(rect.height * ratio));
  if (canvas.width !== width || canvas.height !== height) {
    canvas.width = width;
    canvas.height = height;
  }
  ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
  return { width: rect.width, height: rect.height };
}

function projection(width, height) {
  const pad = { left: 36, right: 25, top: 34, bottom: 30 };
  const scaleX = (width - pad.left - pad.right) / WORLD.width;
  const scaleY = (height - pad.top - pad.bottom) / WORLD.height;
  const scale = Math.min(scaleX, scaleY);
  const offsetX = pad.left + ((width - pad.left - pad.right) - WORLD.width * scale) * 0.5;
  const offsetY = pad.top + ((height - pad.top - pad.bottom) - WORLD.height * scale) * 0.5;
  return {
    point: ({ x, y }) => ({ x: offsetX + x * scale, y: offsetY + (WORLD.height - y) * scale }),
    scale,
    worldY: (screenY) => WORLD.height - (screenY - offsetY) / scale,
  };
}

function syncState() {
  const w = state.wasm;
  state.nodes.forEach((node, index) => {
    node.x = w.ciris_node_x(index);
    node.y = w.ciris_node_y(index);
    node.anchored = Boolean(w.ciris_node_anchored(index));
  });
  state.bonds.forEach((bond, index) => { bond.damage = w.ciris_bond_damage(index); });
}

function drawBackground(width, height, map) {
  const gradient = ctx.createLinearGradient(0, 0, width, height);
  gradient.addColorStop(0, "#1e2d28");
  gradient.addColorStop(0.58, "#14201c");
  gradient.addColorStop(1, "#101916");
  ctx.fillStyle = gradient;
  ctx.fillRect(0, 0, width, height);

  ctx.strokeStyle = "rgba(201, 221, 209, 0.045)";
  ctx.lineWidth = 1;
  for (let x = 0; x <= WORLD.width; x += 0.5) {
    const a = map.point({ x, y: 0.72 });
    const b = map.point({ x, y: WORLD.height - 0.4 });
    ctx.beginPath(); ctx.moveTo(a.x, a.y); ctx.lineTo(b.x, b.y); ctx.stroke();
  }
  for (let y = 0.75; y <= WORLD.height - 0.4; y += 0.5) {
    const a = map.point({ x: 0, y });
    const b = map.point({ x: WORLD.width, y });
    ctx.beginPath(); ctx.moveTo(a.x, a.y); ctx.lineTo(b.x, b.y); ctx.stroke();
  }

  const groundA = map.point({ x: 0, y: 0.75 });
  const groundB = map.point({ x: WORLD.width, y: 0.75 });
  ctx.strokeStyle = "rgba(192, 212, 200, 0.2)";
  ctx.beginPath(); ctx.moveTo(groundA.x, groundA.y); ctx.lineTo(groundB.x, groundB.y); ctx.stroke();
}

function drawAim(map) {
  if (state.launched) return;
  const ball = map.point({ x: state.wasm.ciris_ball_x(), y: state.wasm.ciris_ball_y() });
  const target = map.point({ x: WALL.x + 0.65, y: state.pointerY });
  ctx.save();
  ctx.setLineDash([4, 7]);
  ctx.strokeStyle = "rgba(116, 196, 164, 0.38)";
  ctx.lineWidth = 1;
  ctx.beginPath(); ctx.moveTo(ball.x, ball.y); ctx.lineTo(target.x, target.y); ctx.stroke();
  ctx.setLineDash([]);
  ctx.strokeStyle = "rgba(116, 196, 164, 0.9)";
  ctx.beginPath(); ctx.arc(target.x, target.y, 8, 0, Math.PI * 2); ctx.stroke();
  ctx.beginPath(); ctx.moveTo(target.x - 12, target.y); ctx.lineTo(target.x + 12, target.y); ctx.stroke();
  ctx.beginPath(); ctx.moveTo(target.x, target.y - 12); ctx.lineTo(target.x, target.y + 12); ctx.stroke();
  ctx.restore();
}

function drawWall(map) {
  const xs = state.nodes.map((node) => node.x);
  const ys = state.nodes.map((node) => node.y);
  const slabA = map.point({ x: Math.min(...xs) - 0.15, y: Math.min(...ys) - 0.15 });
  const slabB = map.point({ x: Math.max(...xs) + 0.15, y: Math.max(...ys) + 0.15 });
  const slab = ctx.createLinearGradient(slabA.x, 0, slabB.x, 0);
  slab.addColorStop(0, "rgba(145, 148, 137, 0.11)");
  slab.addColorStop(1, "rgba(111, 120, 110, 0.2)");
  ctx.fillStyle = slab;
  ctx.fillRect(slabA.x, slabB.y, slabB.x - slabA.x, slabA.y - slabB.y);

  for (const bond of state.bonds) {
    const a = map.point(state.nodes[bond.a]);
    const b = map.point(state.nodes[bond.b]);
    if (bond.damage >= 1) {
      const dx = b.x - a.x;
      const dy = b.y - a.y;
      const distance = Math.max(1, Math.hypot(dx, dy));
      const ux = dx / distance;
      const uy = dy / distance;
      const marker = Math.min(7, distance * 0.2);
      ctx.strokeStyle = "rgba(229, 88, 49, 0.95)";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(a.x, a.y);
      ctx.lineTo(a.x + ux * marker, a.y + uy * marker);
      ctx.moveTo(b.x - ux * marker, b.y - uy * marker);
      ctx.lineTo(b.x, b.y);
      ctx.stroke();
      continue;
    }
    ctx.strokeStyle = bond.damage > 0
      ? `rgba(225, 112, 60, ${0.35 + bond.damage * 0.55})`
      : bond.weak ? "rgba(220, 164, 65, 0.5)" : "rgba(170, 184, 172, 0.27)";
    ctx.lineWidth = bond.damage > 0 ? 1.3 : 0.65;
    if (bond.weak && bond.damage === 0) ctx.setLineDash([2, 3]);
    ctx.beginPath(); ctx.moveTo(a.x, a.y); ctx.lineTo(b.x, b.y); ctx.stroke();
    ctx.setLineDash([]);
  }

  const radius = Math.max(1.5, NODE_RADIUS * map.scale * 0.62);
  for (const node of state.nodes) {
    const p = map.point(node);
    ctx.fillStyle = node.anchored ? "rgba(105, 155, 134, 0.72)" : "rgba(198, 201, 188, 0.74)";
    ctx.beginPath(); ctx.arc(p.x, p.y, radius, 0, Math.PI * 2); ctx.fill();
  }
}

function drawBall(map) {
  const p = map.point({ x: state.wasm.ciris_ball_x(), y: state.wasm.ciris_ball_y() });
  const radius = BALL_RADIUS * map.scale;
  ctx.save();
  ctx.shadowColor = "rgba(202, 86, 49, 0.32)";
  ctx.shadowBlur = 24;
  const fill = ctx.createRadialGradient(p.x - radius * 0.34, p.y - radius * 0.38, radius * 0.05, p.x, p.y, radius);
  fill.addColorStop(0, "#f7c588");
  fill.addColorStop(0.28, "#cf693f");
  fill.addColorStop(1, "#742918");
  ctx.fillStyle = fill;
  ctx.beginPath(); ctx.arc(p.x, p.y, radius, 0, Math.PI * 2); ctx.fill();
  ctx.shadowBlur = 0;
  ctx.strokeStyle = "rgba(255, 228, 194, 0.45)";
  ctx.lineWidth = 1;
  ctx.stroke();
  ctx.restore();
}

function updateTelemetry() {
  const w = state.wasm;
  const cracks = w.ciris_cracked_bonds();
  const damage = w.ciris_maximum_damage();
  ui["crack-count"].textContent = cracks.toLocaleString();
  ui["max-damage"].textContent = damage.toFixed(3);
  ui["damage-fill"].style.width = `${damage * 100}%`;
  ui.clock.textContent = `T + ${w.ciris_time().toFixed(3)} s`;
  ui["ball-speed"].textContent = `${w.ciris_ball_speed().toFixed(2)} m/s`;
  ui["contact-force"].textContent = `${w.ciris_peak_contact_force().toFixed(1)} N*`;
  ui["impact-count"].textContent = w.ciris_impact_count();
  document.body.dataset.cracks = String(cracks);
  if (state.launched) {
    ui["stage-status"].textContent = cracks > 0 ? "CRACK FRONT RESOLVED" : "RUST/WASM ADVANCING";
  }
}

function render(now) {
  if (state.wasm) {
    const elapsed = Math.min((now - state.lastFrame) / 1000, 0.05);
    if (state.running) {
      state.wasm.ciris_step(elapsed);
      if (state.wasm.ciris_time() > 4.5) state.running = false;
    }
    syncState();
    const { width, height } = fitCanvas();
    const map = projection(width, height);
    drawBackground(width, height, map);
    drawAim(map);
    drawWall(map);
    drawBall(map);
    updateTelemetry();
  }
  state.lastFrame = now;
  requestAnimationFrame(render);
}

function aimFromPointer(event) {
  if (!state.wasm) return;
  const rect = canvas.getBoundingClientRect();
  const map = projection(rect.width, rect.height);
  state.pointerY = Math.max(WALL.bottom, Math.min(WALL.top, map.worldY(event.clientY - rect.top)));
}

canvas.addEventListener("pointermove", aimFromPointer);
canvas.addEventListener("pointerdown", (event) => {
  if (!state.wasm) return;
  aimFromPointer(event);
  state.wasm.ciris_launch(state.pointerY, Number(speedControl.value));
  state.running = true;
  state.launched = true;
  ui["aim-copy"].classList.add("hidden");
  canvas.setPointerCapture?.(event.pointerId);
});

canvas.addEventListener("keydown", (event) => {
  if (event.key !== "Enter" && event.key !== " ") return;
  event.preventDefault();
  state.wasm?.ciris_launch(state.pointerY, Number(speedControl.value));
  state.running = true;
  state.launched = true;
  ui["aim-copy"].classList.add("hidden");
});

resetButton.addEventListener("click", () => {
  state.wasm?.ciris_reset();
  state.running = false;
  state.launched = false;
  ui["aim-copy"].classList.remove("hidden");
  ui["stage-status"].textContent = "READY TO THROW";
});

speedControl.addEventListener("input", () => { speedOutput.value = Number(speedControl.value).toFixed(1); });

loadWasm().catch((error) => {
  document.body.dataset.engine = "error";
  ui["runtime-status"].textContent = "WASM failed to load";
  ui["stage-status"].textContent = "BUILD WITH ./build-web.sh";
  console.error(error);
});

requestAnimationFrame(render);
