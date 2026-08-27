const stage = document.querySelector("#stage");
const ctx = stage.getContext("2d");
const energyCanvas = document.querySelector("#energy-chart");
const energyCtx = energyCanvas.getContext("2d");
const slider = document.querySelector("#frame-slider");
const playButton = document.querySelector("#play-button");
const playIcon = document.querySelector("#play-icon");

const dom = Object.fromEntries(
  [
    "status-label",
    "frame-label",
    "time-label",
    "scenario-title",
    "scenario-description",
    "scenario-index",
    "particle-count",
    "edge-count",
    "contact-count",
    "mean-speed",
    "energy-total",
    "energy-kinetic",
    "energy-spring",
    "radius-error",
    "constraint-fill",
    "integrator-label",
    "contact-label",
    "boundary-label",
    "step-label",
    "duration-label",
  ].map((id) => [id, document.querySelector(`#${id}`)]),
);

const state = {
  data: null,
  scenario: 0,
  frame: 0,
  playing: true,
  yaw: -0.58,
  pitch: -0.18,
  zoom: 1,
  dragging: false,
  pointer: null,
  lastTick: performance.now(),
  accumulator: 0,
};

function fitCanvas(canvas, context) {
  const ratio = Math.min(window.devicePixelRatio || 1, 2);
  const rect = canvas.getBoundingClientRect();
  const width = Math.max(1, Math.round(rect.width * ratio));
  const height = Math.max(1, Math.round(rect.height * ratio));
  if (canvas.width !== width || canvas.height !== height) {
    canvas.width = width;
    canvas.height = height;
  }
  context.setTransform(ratio, 0, 0, ratio, 0, 0);
  return { width: rect.width, height: rect.height, ratio };
}

function rotate(point) {
  const cy = Math.cos(state.yaw);
  const sy = Math.sin(state.yaw);
  const cp = Math.cos(state.pitch);
  const sp = Math.sin(state.pitch);
  const x1 = cy * point[0] + sy * point[2];
  const z1 = -sy * point[0] + cy * point[2];
  return [x1, cp * point[1] - sp * z1, sp * point[1] + cp * z1];
}

function projector(width, height) {
  const focal = Math.min(width, height) * 1.02 * state.zoom;
  const camera = 3.35;
  const centreX = width * 0.5;
  const centreY = height * 0.48;
  return (point) => {
    const rotated = rotate(point);
    const scale = focal / (camera - rotated[2]);
    return {
      x: centreX + rotated[0] * scale,
      y: centreY - rotated[1] * scale,
      z: rotated[2],
      scale,
    };
  };
}

function speedColor(speed, maxSpeed, alpha = 1) {
  const t = Math.max(0, Math.min(1, speed / Math.max(maxSpeed, 1e-6)));
  const stops = [
    [70, 229, 207],
    [113, 204, 232],
    [244, 174, 98],
  ];
  const segment = t < 0.5 ? 0 : 1;
  const local = segment === 0 ? t * 2 : (t - 0.5) * 2;
  const a = stops[segment];
  const b = stops[segment + 1];
  const rgb = a.map((value, i) => Math.round(value + (b[i] - value) * local));
  return `rgba(${rgb[0]}, ${rgb[1]}, ${rgb[2]}, ${alpha})`;
}

function drawBackdrop(width, height, project) {
  const centre = project([0, 0, 0]);
  const rim = project([1, 0, 0]);
  const radius = Math.abs(rim.x - centre.x) * 1.025;

  const halo = ctx.createRadialGradient(
    centre.x - radius * 0.24,
    centre.y - radius * 0.28,
    radius * 0.08,
    centre.x,
    centre.y,
    radius * 1.25,
  );
  halo.addColorStop(0, "rgba(44, 115, 103, 0.19)");
  halo.addColorStop(0.56, "rgba(10, 37, 39, 0.45)");
  halo.addColorStop(1, "rgba(2, 10, 12, 0)");
  ctx.fillStyle = halo;
  ctx.beginPath();
  ctx.arc(centre.x, centre.y, radius * 1.27, 0, Math.PI * 2);
  ctx.fill();

  const sphere = ctx.createRadialGradient(
    centre.x - radius * 0.36,
    centre.y - radius * 0.4,
    radius * 0.06,
    centre.x,
    centre.y,
    radius,
  );
  sphere.addColorStop(0, "rgba(24, 64, 61, 0.72)");
  sphere.addColorStop(0.62, "rgba(8, 28, 31, 0.84)");
  sphere.addColorStop(1, "rgba(2, 11, 14, 0.96)");
  ctx.fillStyle = sphere;
  ctx.beginPath();
  ctx.arc(centre.x, centre.y, radius, 0, Math.PI * 2);
  ctx.fill();
  ctx.strokeStyle = "rgba(99, 215, 190, 0.22)";
  ctx.lineWidth = 1;
  ctx.stroke();

  const shadow = ctx.createRadialGradient(
    centre.x,
    centre.y + radius * 1.11,
    0,
    centre.x,
    centre.y + radius * 1.11,
    radius * 0.72,
  );
  shadow.addColorStop(0, "rgba(0, 0, 0, 0.42)");
  shadow.addColorStop(1, "rgba(0, 0, 0, 0)");
  ctx.save();
  ctx.scale(1, 0.18);
  ctx.fillStyle = shadow;
  ctx.beginPath();
  ctx.arc(centre.x, (centre.y + radius * 1.17) / 0.18, radius * 0.78, 0, Math.PI * 2);
  ctx.fill();
  ctx.restore();

  return radius;
}

function drawGrid(project) {
  const drawLine = (points, alpha) => {
    const projected = points.map(project);
    for (let i = 1; i < projected.length; i += 1) {
      const front = (projected[i - 1].z + projected[i].z) * 0.5;
      ctx.strokeStyle = `rgba(91, 164, 150, ${front > 0 ? alpha : alpha * 0.27})`;
      ctx.lineWidth = 0.65;
      ctx.beginPath();
      ctx.moveTo(projected[i - 1].x, projected[i - 1].y);
      ctx.lineTo(projected[i].x, projected[i].y);
      ctx.stroke();
    }
  };

  for (let latitude = -60; latitude <= 60; latitude += 30) {
    const phi = (latitude * Math.PI) / 180;
    const points = [];
    for (let step = 0; step <= 96; step += 1) {
      const theta = (step / 96) * Math.PI * 2;
      points.push([
        Math.cos(phi) * Math.cos(theta),
        Math.sin(phi),
        Math.cos(phi) * Math.sin(theta),
      ]);
    }
    drawLine(points, latitude === 0 ? 0.17 : 0.1);
  }

  for (let meridian = 0; meridian < 12; meridian += 1) {
    const theta = (meridian / 12) * Math.PI * 2;
    const points = [];
    for (let step = 0; step <= 64; step += 1) {
      const phi = -Math.PI / 2 + (step / 64) * Math.PI;
      points.push([
        Math.cos(phi) * Math.cos(theta),
        Math.sin(phi),
        Math.cos(phi) * Math.sin(theta),
      ]);
    }
    drawLine(points, 0.08);
  }
}

function drawSimulation(frame, simulation, project) {
  const projected = frame.positions.map(project);
  const maxSpeed = Math.max(simulation.summary.peak_speed * 0.78, 0.1);
  const edges = state.data.edges
    .map(([a, b]) => ({ a, b, depth: (projected[a].z + projected[b].z) * 0.5 }))
    .sort((a, b) => a.depth - b.depth);

  for (const edge of edges) {
    const a = projected[edge.a];
    const b = projected[edge.b];
    const front = Math.max(0, Math.min(1, (edge.depth + 1) * 0.5));
    ctx.strokeStyle = `rgba(70, 208, 184, ${0.055 + front * 0.2})`;
    ctx.lineWidth = 0.45 + front * 0.55;
    ctx.beginPath();
    ctx.moveTo(a.x, a.y);
    ctx.lineTo(b.x, b.y);
    ctx.stroke();
  }

  const particles = projected
    .map((point, index) => ({ ...point, index }))
    .sort((a, b) => a.z - b.z);
  for (const point of particles) {
    const speed = frame.speeds[point.index];
    const depth = Math.max(0, Math.min(1, (point.z + 1) * 0.5));
    const radius = Math.max(2.1, point.scale * state.data.meta.particle_radius * 0.72);
    const color = speedColor(speed, maxSpeed, 0.52 + 0.48 * depth);
    const glow = ctx.createRadialGradient(
      point.x - radius * 0.28,
      point.y - radius * 0.34,
      radius * 0.08,
      point.x,
      point.y,
      radius,
    );
    glow.addColorStop(0, "rgba(244, 255, 251, 0.98)");
    glow.addColorStop(0.27, color);
    glow.addColorStop(1, speedColor(speed, maxSpeed, 0.12));
    ctx.fillStyle = glow;
    ctx.shadowColor = speedColor(speed, maxSpeed, 0.45);
    ctx.shadowBlur = depth > 0.55 ? 9 : 3;
    ctx.beginPath();
    ctx.arc(point.x, point.y, radius, 0, Math.PI * 2);
    ctx.fill();
  }
  ctx.shadowBlur = 0;
}

function drawEnergyChart(simulation, frameIndex) {
  const { width, height } = fitCanvas(energyCanvas, energyCtx);
  energyCtx.clearRect(0, 0, width, height);
  const frames = simulation.frames;
  const energies = frames.map((frame) => frame.total_energy);
  const min = Math.min(...energies);
  const max = Math.max(...energies);
  const range = Math.max(max - min, Math.abs(max) * 0.015, 1e-9);
  const pad = 4;

  energyCtx.strokeStyle = "rgba(111, 160, 149, 0.12)";
  energyCtx.lineWidth = 1;
  for (let row = 1; row < 4; row += 1) {
    const y = (height * row) / 4;
    energyCtx.beginPath();
    energyCtx.moveTo(0, y);
    energyCtx.lineTo(width, y);
    energyCtx.stroke();
  }

  const gradient = energyCtx.createLinearGradient(0, 0, width, 0);
  gradient.addColorStop(0, "rgba(85, 241, 204, 0.45)");
  gradient.addColorStop(1, "rgba(243, 173, 98, 0.95)");
  energyCtx.strokeStyle = gradient;
  energyCtx.lineWidth = 1.5;
  energyCtx.beginPath();
  energies.forEach((energy, index) => {
    const x = pad + (index / Math.max(1, energies.length - 1)) * (width - pad * 2);
    const y = height - pad - ((energy - min) / range) * (height - pad * 2);
    if (index === 0) energyCtx.moveTo(x, y);
    else energyCtx.lineTo(x, y);
  });
  energyCtx.stroke();

  const markerX = pad + (frameIndex / Math.max(1, frames.length - 1)) * (width - pad * 2);
  energyCtx.strokeStyle = "rgba(231, 242, 239, 0.48)";
  energyCtx.beginPath();
  energyCtx.moveTo(markerX, 0);
  energyCtx.lineTo(markerX, height);
  energyCtx.stroke();
}

function updateReadout(simulation, frame) {
  const frameCount = simulation.frames.length;
  dom["frame-label"].textContent = `FRAME ${String(state.frame + 1).padStart(3, "0")} / ${frameCount}`;
  dom["time-label"].textContent = `T = ${frame.time.toFixed(3)}`;
  dom["scenario-title"].textContent = simulation.title;
  dom["scenario-description"].textContent = simulation.description;
  dom["scenario-index"].textContent = `${String(state.scenario + 1).padStart(2, "0")} / ${String(state.data.simulations.length).padStart(2, "0")}`;
  dom["particle-count"].textContent = state.data.meta.particle_count;
  dom["edge-count"].textContent = state.data.meta.edge_count;
  dom["contact-count"].textContent = frame.contact_events.toLocaleString();
  dom["mean-speed"].textContent = frame.mean_speed.toFixed(4);
  dom["energy-total"].textContent = frame.total_energy.toFixed(4);
  dom["energy-kinetic"].textContent = frame.kinetic_energy.toFixed(3);
  dom["energy-spring"].textContent = frame.spring_energy.toFixed(3);
  dom["radius-error"].textContent = frame.max_radius_error.toExponential(2);
  dom["constraint-fill"].style.width = `${Math.min(100, 4 + Math.abs(Math.log10(Math.max(frame.max_radius_error, 1e-18))) * 1.7)}%`;
  dom["integrator-label"].textContent = state.data.meta.integrator;
  dom["contact-label"].textContent = state.data.meta.contact_solver;
  dom["boundary-label"].textContent = state.data.meta.surface_constraint;
  dom["step-label"].textContent = `step ${frame.step.toLocaleString()}`;
  const finalFrame = simulation.frames[frameCount - 1];
  dom["duration-label"].textContent = `${finalFrame.time.toFixed(3)} seconds simulated`;
  slider.max = frameCount - 1;
  slider.value = state.frame;
  slider.style.setProperty("--progress", `${(state.frame / (frameCount - 1)) * 100}%`);
  drawEnergyChart(simulation, state.frame);
}

function render() {
  if (!state.data) return;
  const { width, height } = fitCanvas(stage, ctx);
  ctx.clearRect(0, 0, width, height);
  const project = projector(width, height);
  drawBackdrop(width, height, project);
  drawGrid(project);
  const simulation = state.data.simulations[state.scenario];
  const frame = simulation.frames[state.frame];
  drawSimulation(frame, simulation, project);
  updateReadout(simulation, frame);
}

function setScenario(index) {
  state.scenario = index;
  state.frame = 0;
  state.accumulator = 0;
  document.querySelectorAll(".scenario-button").forEach((button, buttonIndex) => {
    button.classList.toggle("active", buttonIndex === index);
    button.setAttribute("aria-pressed", String(buttonIndex === index));
  });
  render();
}

function buildScenarioButtons() {
  const strip = document.querySelector("#scenario-strip");
  strip.replaceChildren();
  state.data.simulations.forEach((simulation, index) => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = `scenario-button${index === 0 ? " active" : ""}`;
    button.setAttribute("aria-pressed", String(index === 0));
    button.innerHTML = `<b>${String(index + 1).padStart(2, "0")} · ${simulation.title}</b><span>${simulation.summary.total_contact_events.toLocaleString()} contacts · peak v ${simulation.summary.peak_speed.toFixed(3)}</span>`;
    button.addEventListener("click", () => setScenario(index));
    strip.append(button);
  });
}

function tick(now) {
  if (state.data && state.playing) {
    const simulation = state.data.simulations[state.scenario];
    const frameInterval = state.data.meta.dt * state.data.meta.steps_per_frame * 1000;
    state.accumulator += Math.min(100, now - state.lastTick);
    if (state.accumulator >= frameInterval) {
      const advance = Math.floor(state.accumulator / frameInterval);
      state.frame = (state.frame + advance) % simulation.frames.length;
      state.accumulator %= frameInterval;
      state.yaw += 0.0007 * advance;
      render();
    }
  }
  state.lastTick = now;
  requestAnimationFrame(tick);
}

playButton.addEventListener("click", () => {
  state.playing = !state.playing;
  playIcon.textContent = state.playing ? "Ⅱ" : "▶";
  playButton.setAttribute("aria-label", state.playing ? "Pause simulation" : "Play simulation");
});

slider.addEventListener("input", () => {
  state.frame = Number(slider.value);
  state.playing = false;
  playIcon.textContent = "▶";
  playButton.setAttribute("aria-label", "Play simulation");
  render();
});

document.querySelector("#reset-view").addEventListener("click", () => {
  state.yaw = -0.58;
  state.pitch = -0.18;
  state.zoom = 1;
  render();
});

stage.addEventListener("pointerdown", (event) => {
  state.dragging = true;
  state.pointer = [event.clientX, event.clientY];
  stage.classList.add("dragging");
  stage.setPointerCapture(event.pointerId);
});

stage.addEventListener("pointermove", (event) => {
  if (!state.dragging) return;
  const [lastX, lastY] = state.pointer;
  state.yaw += (event.clientX - lastX) * 0.008;
  state.pitch = Math.max(-1.25, Math.min(1.25, state.pitch + (event.clientY - lastY) * 0.008));
  state.pointer = [event.clientX, event.clientY];
  render();
});

function stopDragging(event) {
  state.dragging = false;
  state.pointer = null;
  stage.classList.remove("dragging");
  if (event.pointerId !== undefined && stage.hasPointerCapture(event.pointerId)) {
    stage.releasePointerCapture(event.pointerId);
  }
}

stage.addEventListener("pointerup", stopDragging);
stage.addEventListener("pointercancel", stopDragging);
stage.addEventListener(
  "wheel",
  (event) => {
    event.preventDefault();
    state.zoom = Math.max(0.72, Math.min(1.42, state.zoom - event.deltaY * 0.0007));
    render();
  },
  { passive: false },
);

window.addEventListener("resize", render);

fetch("simulation.json")
  .then((response) => {
    if (!response.ok) throw new Error(`frame bundle request failed: ${response.status}`);
    return response.json();
  })
  .then((data) => {
    state.data = data;
    dom["status-label"].textContent = "RUST FRAME BUNDLE ONLINE";
    buildScenarioButtons();
    render();
    requestAnimationFrame(tick);
  })
  .catch((error) => {
    dom["status-label"].textContent = "FRAME BUNDLE ERROR";
    dom["scenario-title"].textContent = "Unable to load simulation.json";
    dom["scenario-description"].textContent = error.message;
    console.error(error);
  });
