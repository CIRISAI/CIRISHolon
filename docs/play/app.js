// THE ACUITY SANDBOX — a water toy.
//
// WHAT THIS IS NOT. It is not an instrument and it makes no claim about water. The blobs are
// invented: a cheap 2D kernel with no physical content, tuned to look wet. The molecules
// inside the focus ring ARE run by the same engine the instrument uses, under a real wall
// law, and they are still a dozen molecules in a vacuum at an invented temperature — which is
// not water either. Nothing on this page may be cited, and `wall.mjs` fails the build if
// anything here ever points at the record, in either direction. It caught this very header.
//
// WHAT IT IS FOR. One idea, made playable: THE COST IS GOVERNED BY WHERE YOU LOOK. Everything
// outside the ring is a blob costing almost nothing. Everything inside is real and expensive.
// Move the ring and the bill follows it. That is the acuity idea the programme's edge axis is
// built on, with the science taken out and the mechanic left in.

const CORE_PX = 13;                  // the blob soft core, px — the layer's natural spacing
// THE WORLD SCALE, and it is the one number here that has to be chosen rather than invented:
// the blob layer packs at about its soft-core diameter, so mapping that spacing onto water's
// own oxygen-oxygen separation is what stops the promoted molecules being seeded on top of
// each other. At 0.115 they were seeded 3.6 bohr apart — far inside the repulsive wall — and
// the engine did exactly what it should: it blew them apart.
const BOHR_PER_PX = 5.3 / CORE_PX;   // water's O-O nearest neighbour over the blob spacing
const N_BLOBS = 520;                 // the invented layer
// THE BUDGET, set by measurement and not by nerve: the HUD showed a force pass under 0.05 ms
// at fourteen molecules, so fourteen was costing nothing and hiding the point of the toy. The
// number to show a player is the one where the bill starts to be visible. Measured again once
// the engine was actually stepping: 0.033 ms a pass at twelve, twenty-four and forty-eight
// molecules alike — fixed overhead dominates at this size, so the cap is the ring's to set.
const REAL_CAP = 96;                 // molecules the engine carries at once — a BUDGET, not a limit of the law
// The ring is a MAGNIFIER as well as a budget: a molecule is 1.94 bohr across and the world
// scale puts that at five pixels, which reads as a speck. Drawing the real layer magnified
// about the focus centre is the one place this toy deliberately lies about geometry, and it
// lies in the direction of letting you see what is actually there.
const MAG = 3.4;
const RESEED_EVERY = 24;             // frames between membership reviews; re-seeding costs a reset
const PIN_R = 1.9435738400;          // the engine's own pinned monomer, bohr
const PIN_THETA = 1.6887434037;
const LAW = {                        // the served wall law's own numbers, pasted — a toy may paste
  a: 9.480487358002e2, b: 2.4, p: 1.7643681255e1, c: 2.36, c6: 0,
  a_oh: 2.258605362549e1, b_oh: 2.2, a_hh: 1.525046249564, b_hh: 1.75,
  p_hh: 7.451162320433e-1, c_hh: 2.34, p_ct: 0, c_ct: 0, m_ct: 0, k_ct: 0, lambda_ct: 0,
  r_cut: 14.0,
};

const $ = (id) => document.getElementById(id);
const canvas = $("stage"), ctx = canvas.getContext("2d", { alpha: false });
let W = 0, H = 0, DPR = 1;
function resize() {
  DPR = Math.min(window.devicePixelRatio || 1, 2);
  W = canvas.clientWidth; H = canvas.clientHeight;
  canvas.width = Math.round(W * DPR); canvas.height = Math.round(H * DPR);
  ctx.setTransform(DPR, 0, 0, DPR, 0, 0);
}
window.addEventListener("resize", resize);

// ---------------------------------------------------------------- the invented layer

const blobs = [];
function seedBlobs() {
  blobs.length = 0;
  for (let i = 0; i < N_BLOBS; i++) {
    blobs.push({ x: W * (0.18 + 0.64 * Math.random()), y: H * (0.35 + 0.55 * Math.random()),
                 vx: 0, vy: 0, real: false });
  }
}
// A cheap cohesive kernel on a grid. No physical content whatever: a soft core, a shallow
// pull, a viscosity, and walls. It exists to look wet and to be nearly free.
const CELL = 26, GRAV = 620, CORE = CORE_PX, PULL = 10.5, VISC = 0.10, WALL_E = 0.42;
function stepBlobs(dt) {
  const grid = new Map();
  const key = (cx, cy) => cx * 100003 + cy;
  for (const b of blobs) {
    if (b.real) continue;
    const k = key(Math.floor(b.x / CELL), Math.floor(b.y / CELL));
    let a = grid.get(k); if (!a) grid.set(k, a = []); a.push(b);
  }
  for (const b of blobs) {
    if (b.real) continue;
    b.vy += GRAV * dt;
    const cx = Math.floor(b.x / CELL), cy = Math.floor(b.y / CELL);
    for (let i = -1; i <= 1; i++) for (let j = -1; j <= 1; j++) {
      const a = grid.get(key(cx + i, cy + j)); if (!a) continue;
      for (const o of a) {
        if (o === b) continue;
        let dx = o.x - b.x, dy = o.y - b.y;
        const d2 = dx * dx + dy * dy;
        if (d2 > CELL * CELL || d2 < 1e-6) continue;
        const d = Math.sqrt(d2);
        const f = d < CORE ? -((CORE - d) / CORE) * 2600 : ((CELL - d) / CELL) * PULL;
        b.vx += (dx / d) * f * dt; b.vy += (dy / d) * f * dt;
        b.vx += (o.vx - b.vx) * VISC * dt; b.vy += (o.vy - b.vy) * VISC * dt;
      }
    }
  }
  const top = 52;
  for (const b of blobs) {
    if (b.real) continue;
    b.x += b.vx * dt; b.y += b.vy * dt;
    if (b.x < 8) { b.x = 8; b.vx = -b.vx * WALL_E; }
    if (b.x > W - 8) { b.x = W - 8; b.vx = -b.vx * WALL_E; }
    if (b.y < top) { b.y = top; b.vy = -b.vy * WALL_E; }
    if (b.y > H - 8) { b.y = H - 8; b.vy = -b.vy * WALL_E; b.vx *= 0.94; }
  }
}

// ---------------------------------------------------------------- the real layer

const Engine = { w: null, ok: false, molecules: 0, passMs: 0, fsPerSecond: 0, stepped: 0, note: "booting" };

async function boot() {
  try {
    const r = await fetch("holon_render.wasm");
    if (!r.ok) throw new Error(`HTTP ${r.status}`);
    const { instance } = await WebAssembly.instantiate(await r.arrayBuffer(), {});
    const w = instance.exports;
    for (const n of ["holon_reset", "holon_set_atom_species", "holon_set_position_3d",
                     "holon_set_boundary", "holon_set_seam", "holon_step_frame", "holon_steps",
                     "holon_bank_slot", "holon_bank_table_begin", "holon_bank_table_knot",
                     "holon_bank_table_knot_curvature", "holon_bank_table_finish",
                     "holon_atom_count", "holon_atom_x", "holon_atom_y", "holon_atom_z", "holon_rebase"]) {
      if (typeof w[n] !== "function") throw new Error(`the artifact has no ${n}`);
    }
    // THE PAIR CURVES, AND THE ORDER THEY NEED. Without them the bank has no curve, the
    // timescale derives no step, and `holon_step_frame` returns having advanced nothing —
    // which is exactly what this toy did on its first run: `holon_steps()` read 0 after two
    // hundred calls and the "real" molecules were a still life. The engine was right and the
    // page was wrong, and the page said "live" while it was wrong, which was worse.
    //
    // Two things had to be learned from the engine rather than assumed. The BANK REGISTERS A
    // PAIR ONLY WHEN THE SCENE HOLDS BOTH SPECIES, so a scene has to exist before a slot does
    // — `holon_bank_slot(8,1)` answers -1 to a page that asks too early. And H–H is on the
    // SOLVE-HERE side of the engine's own split: no file ships for it and the browser is
    // expected to generate it, which is one call and about a millisecond.
    w.holon_reset(3);
    w.holon_set_atom_species(0, 8); w.holon_set_atom_species(1, 1); w.holon_set_atom_species(2, 1);
    if (w.holon_bank_generate_pair(1, 1, 48) !== 1) throw new Error("the browser could not generate the H-H curve");
    for (const [pair, file] of [[[8, 1], "tables/HO.json"], [[8, 8], "tables/O2.json"]]) {
      const jr = await fetch(file);
      if (!jr.ok) throw new Error(`${file}: HTTP ${jr.status}`);
      const j = await jr.json();
      const slot = w.holon_bank_slot(pair[0], pair[1]);
      if (slot < 0) throw new Error(`${file}: not a registered pair`);
      const n = j.R_grid_bohr.length;
      if (w.holon_bank_table_begin(slot, n) !== 1) throw new Error(`${file}: the bank refused ${n} knots`);
      for (let i = 0; i < n; i++) {
        w.holon_bank_table_knot(slot, i, j.R_grid_bohr[i], j.E_hartree[i], j.F_hartree_per_bohr[i]);
        w.holon_bank_table_knot_curvature(slot, i, j.E2_hartree_per_bohr2[i]);
      }
      const route = j.solver_route === "determinant" ? 1 : j.solver_route === "DMRG" ? 2 : 0;
      const code = w.holon_bank_table_finish(slot, j.R_e, j.D_e, j.E_asymptote, route,
        j.species.n_determinants, j.species.n_basis, j.uncertainty_hartree, j.exact_in_model ? 1 : 0);
      if (code !== 1) throw new Error(`${file}: the bank refused the curve (code ${code})`);
    }
    // AND PROVE IT MOVES, before the page is allowed to say "live". A banner that says live
    // over a still life is the one lie this toy could still tell.
    w.holon_set_boundary(1);
    w.holon_set_seam(1, LAW.a, LAW.b, LAW.p, LAW.c, LAW.c6, LAW.a_oh, LAW.b_oh, LAW.a_hh,
                     LAW.b_hh, LAW.p_hh, LAW.c_hh, LAW.p_ct, LAW.c_ct, LAW.m_ct, LAW.k_ct,
                     LAW.lambda_ct, LAW.r_cut);
    w.holon_set_position_3d(0, 0, 0, 0);
    w.holon_set_position_3d(1, 1.5, 1.2, 0);
    w.holon_set_position_3d(2, 1.5, -1.2, 0);
    w.holon_rebase();
    const s0 = w.holon_steps(), y0 = w.holon_atom_y(1);
    for (let k = 0; k < 40; k++) w.holon_step_frame(1);
    if (w.holon_steps() - s0 < 40) throw new Error("the engine accepted the scene and advanced nothing");
    if (Math.abs(w.holon_atom_y(1) - y0) < 1e-12) throw new Error("the engine stepped and no atom moved");
    Engine.w = w; Engine.ok = true;
    Engine.note = `live, dt ${w.holon_dt().toFixed(4)} au`;
  } catch (e) {
    Engine.note = `no engine: ${e.message}`;   // the toy still runs; the ring just stays empty
  }
  $("h-engine").textContent = Engine.note;
  return Engine.ok;
}

/// Seed the engine with one molecule per promoted blob: 3 atoms each, oxygen first, at the
/// engine's own pinned monomer geometry with a random orientation in the viewing plane.
function seedReal(promoted) {
  const w = Engine.w, n = promoted.length;
  if (!w || n === 0) { Engine.molecules = 0; return; }
  w.holon_reset(3 * n);
  w.holon_set_boundary(1);                       // OPEN: the image rule refuses a small wrap
  w.holon_set_seam(1, LAW.a, LAW.b, LAW.p, LAW.c, LAW.c6, LAW.a_oh, LAW.b_oh, LAW.a_hh,
                   LAW.b_hh, LAW.p_hh, LAW.c_hh, LAW.p_ct, LAW.c_ct, LAW.m_ct, LAW.k_ct,
                   LAW.lambda_ct, LAW.r_cut);
  const half = PIN_THETA / 2;
  const placed = [];
  for (let m = 0; m < n; m++) {
    const b = promoted[m];
    let ox = b.x * BOHR_PER_PX, oy = b.y * BOHR_PER_PX;
    // A MOLECULE MAY NOT BE SEEDED INSIDE ANOTHER'S WALL. The invented layer has no idea what
    // 4.6 bohr means, so two blobs can be arbitrarily close; handing that to the engine is
    // handing it an overlap it will answer with an explosion. Nudge, do not drop: the count
    // the HUD shows has to stay the count the engine is carrying.
    for (let guard = 0; guard < 24; guard++) {
      let worst = null, wd = 1e9;
      for (const q of placed) { const d = Math.hypot(ox - q[0], oy - q[1]); if (d < 4.6 && d < wd) { wd = d; worst = q; } }
      if (!worst) break;
      const a = Math.atan2(oy - worst[1], ox - worst[0]) || (m * 2.399);
      ox = worst[0] + Math.cos(a) * 4.8; oy = worst[1] + Math.sin(a) * 4.8;
    }
    placed.push([ox, oy]);
    const th = Math.random() * Math.PI * 2;
    w.holon_set_atom_species(3 * m, 8);
    w.holon_set_position_3d(3 * m, ox, oy, 0);
    for (let h = 0; h < 2; h++) {
      const a = th + (h ? -half : half);
      w.holon_set_atom_species(3 * m + 1 + h, 1);
      w.holon_set_position_3d(3 * m + 1 + h, ox + PIN_R * Math.cos(a), oy + PIN_R * Math.sin(a), 0);
    }
  }
  w.holon_rebase();
  // what the engine CARRIES, not what we asked for: if the two ever differ, every index below
  // is reading somebody else's atom, which is how a bond gets drawn to the origin.
  Engine.molecules = Math.min(n, Math.floor(w.holon_atom_count() / 3));
  if (Engine.molecules !== n) Engine.note = `engine carried ${Engine.molecules} of ${n}`;
}

function stepReal() {
  const w = Engine.w;
  if (!w || Engine.molecules === 0) { Engine.passMs = 0; return; }
  const t0 = performance.now();
  const before = w.holon_steps();
  w.holon_step_frame(1);
  Engine.stepped = w.holon_steps() - before;      // 0 means the engine advanced NOTHING
  Engine.passMs = performance.now() - t0;
}

function readReal() {
  const w = Engine.w, out = [];
  if (!w || Engine.molecules === 0) return out;
  for (let m = 0; m < Engine.molecules; m++) {
    const s = [];
    for (let k = 0; k < 3; k++) {
      const i = 3 * m + k;
      s.push([w.holon_atom_x(i) / BOHR_PER_PX, w.holon_atom_y(i) / BOHR_PER_PX]);
    }
    out.push(s);
  }
  return out;
}

// ---------------------------------------------------------------- the focus, and the mechanic

const focus = { x: 0, y: 0, r: 150, held: false };
canvas.addEventListener("pointermove", (e) => { focus.x = e.clientX; focus.y = e.clientY; });
canvas.addEventListener("pointerdown", () => { focus.held = true; });
window.addEventListener("pointerup", () => { focus.held = false; });
canvas.addEventListener("wheel", (e) => {
  e.preventDefault();
  focus.r = Math.max(44, Math.min(240, focus.r + (e.deltaY > 0 ? 10 : -10)));
}, { passive: false });

/// THE ACUITY REVIEW. Blobs inside the ring are promoted, nearest first, up to the budget;
/// the promoted ones stop being blobs and become the engine's. Everything demoted is handed
/// its molecule's centre back and rejoins the invented layer — which is a RECONSTRUCTION in
/// the same sense `holon-runtime` means it, and about as lossy.
function reviewFocus() {
  const sites = readReal();
  const wasReal = blobs.filter((b) => b.real);
  wasReal.forEach((b, m) => { if (sites[m]) { b.x = sites[m][0][0]; b.y = sites[m][0][1]; b.vx = b.vy = 0; } });
  for (const b of blobs) b.real = false;
  const inside = blobs
    .map((b) => ({ b, d: Math.hypot(b.x - focus.x, b.y - focus.y) }))
    .filter((e) => e.d <= focus.r)
    .sort((a, z) => a.d - z.d)
    .slice(0, REAL_CAP);
  for (const e of inside) e.b.real = true;
  seedReal(inside.map((e) => e.b));
}

// ---------------------------------------------------------------- the draw

/// Screen position of a real site: magnified about the focus centre. See MAG.
function mag(p) { return [focus.x + (p[0] - focus.x) * MAG, focus.y + (p[1] - focus.y) * MAG]; }

function draw(sites) {
  ctx.fillStyle = "#0a0f18"; ctx.fillRect(0, 0, W, H);
  ctx.globalCompositeOperation = "lighter";
  for (const b of blobs) {
    if (b.real) continue;
    const g = ctx.createRadialGradient(b.x, b.y, 0, b.x, b.y, 15);
    g.addColorStop(0, "rgba(70,130,200,0.50)"); g.addColorStop(1, "rgba(40,90,160,0)");
    ctx.fillStyle = g; ctx.beginPath(); ctx.arc(b.x, b.y, 15, 0, 6.2832); ctx.fill();
  }
  ctx.globalCompositeOperation = "source-over";
  // the hydrogen bonds the lens would count, drawn only as a hint and counted by nobody
  ctx.strokeStyle = "rgba(150,210,255,0.34)"; ctx.lineWidth = 1.4;
  for (let i = 0; i < sites.length; i++) for (let j = 0; j < sites.length; j++) {
    if (i === j) continue;
    for (let h = 1; h < 3; h++) {
      const d = Math.hypot(sites[i][h][0] - sites[j][0][0], sites[i][h][1] - sites[j][0][1]) * BOHR_PER_PX;
      if (d < 4.2) { const a = mag(sites[i][h]), b = mag(sites[j][0]);
                     ctx.beginPath(); ctx.moveTo(a[0], a[1]); ctx.lineTo(b[0], b[1]); ctx.stroke(); }
    }
  }
  for (const s of sites) {
    const o = mag(s[0]), h1 = mag(s[1]), h2 = mag(s[2]);
    ctx.strokeStyle = "rgba(220,235,255,0.6)"; ctx.lineWidth = 2.5;
    for (const h of [h1, h2]) { ctx.beginPath(); ctx.moveTo(o[0], o[1]); ctx.lineTo(h[0], h[1]); ctx.stroke(); }
    ctx.fillStyle = "#ff6b6b"; ctx.beginPath(); ctx.arc(o[0], o[1], 6.5, 0, 6.2832); ctx.fill();
    ctx.fillStyle = "#e8f0ff";
    for (const h of [h1, h2]) { ctx.beginPath(); ctx.arc(h[0], h[1], 3.6, 0, 6.2832); ctx.fill(); }
  }
  ctx.strokeStyle = focus.held ? "rgba(95,179,255,0.95)" : "rgba(95,179,255,0.5)";
  ctx.lineWidth = focus.held ? 2.5 : 1.5;
  ctx.beginPath(); ctx.arc(focus.x, focus.y, focus.r, 0, 6.2832); ctx.stroke();
}

// ---------------------------------------------------------------- the loop

let frame = 0, last = performance.now(), fps = 60, fsThisSecond = 0, secondMark = last;
const STEP_FS = 0.0260630187;     // the engine's own step at this law, femtoseconds

function tick(now) {
  const dt = Math.min((now - last) / 1000, 0.033); last = now;
  fps += ((1 / Math.max(dt, 1e-4)) - fps) * 0.1;
  if (frame % RESEED_EVERY === 0) reviewFocus();
  stepBlobs(dt);
  stepReal();
  if (Engine.molecules > 0 && Engine.stepped > 0) fsThisSecond += STEP_FS * Engine.stepped;
  const sites = readReal();
  draw(sites);
  if (now - secondMark >= 1000) { Engine.fsPerSecond = fsThisSecond; fsThisSecond = 0; secondMark = now; }
  $("h-focus").textContent = `${Math.round(focus.r)} px`;
  $("h-real").textContent = `${Engine.molecules} of ${REAL_CAP}`;
  $("h-blobs").textContent = `${blobs.filter((b) => !b.real).length}`;
  $("h-pass").textContent = Engine.molecules ? `${Engine.passMs.toFixed(1)} ms` : "— ms";
  $("h-fs").textContent = `${Engine.fsPerSecond.toFixed(2)} fs/s`;
  $("h-fps").textContent = `${fps.toFixed(0)} fps`;
  frame++;
  requestAnimationFrame(tick);
}

window.__toy = { Engine, blobs, focus };   // a handle for inspecting the running toy
resize(); seedBlobs();
focus.x = W / 2; focus.y = H * 0.62;
await boot();
requestAnimationFrame(tick);
