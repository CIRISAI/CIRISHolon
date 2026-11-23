// The synthetic scene of test.html and test-webgpu.html, shared so the two backends draw the
// same records with the same camera and the same panels. Moved out of test.html unchanged
// (test.html's behaviour is the same); the one addition is overlay()'s optional `verdict`,
// which test.html does not pass, so its panel still says PASS and "gl error 0".
import { KIND, INSTANCE_BYTES, PARAMS_NEUTRAL, COLOUR, PARAM, VARIANT_CASING, DISC_CASING, DISC_CANOPY_FIELD, VARIANT_ROUND, VARIANT_ORIENT, MATERIAL, GLYPH, VARIANT_FLOW } from './draw.js';
import { STYLE } from './text.js';

// ---- records ----
export class Batch {
  constructor(kind, offset, coarse, screen) {
    this.kind = kind; this.offset = offset || [0, 0, 0]; this.coarse = !!coarse; this.screen = !!screen;
    this.count = 0; this.buf = new ArrayBuffer(64 * INSTANCE_BYTES); this.dv = new DataView(this.buf);
    this.bytes = new Uint8Array(this.buf, 0, 0);
  }
  clear() { this.count = 0; this.bytes = new Uint8Array(this.buf, 0, 0); return this; }
  add(pos, size, aux, angle, rgba, flags, variant) {
    if ((this.count + 1) * INSTANCE_BYTES > this.buf.byteLength) {
      const nb = new ArrayBuffer(this.buf.byteLength * 2);
      new Uint8Array(nb).set(new Uint8Array(this.buf));
      this.buf = nb; this.dv = new DataView(nb);
    }
    const o = this.count * INSTANCE_BYTES, dv = this.dv;
    for (let i = 0; i < 3; i++) {
      dv.setFloat32(o + 4 * i, pos[i], true);
      dv.setFloat32(o + 12 + 4 * i, size[i] || 0, true);
      dv.setFloat32(o + 24 + 4 * i, aux ? aux[i] : 0, true);
    }
    dv.setFloat32(o + 36, angle || 0, true);
    for (let i = 0; i < 4; i++) dv.setUint8(o + 40 + i, rgba[i]);
    dv.setUint32(o + 44, 0, true);
    dv.setUint8(o + 48, this.kind); dv.setUint8(o + 49, 0); dv.setUint8(o + 50, flags || 0); dv.setUint8(o + 51, variant || 0);
    this.count++;
    this.bytes = new Uint8Array(this.buf, 0, this.count * INSTANCE_BYTES);
    return this;
  }
}

export function scene() {
  const OFF = [100, 200, 0];           // every batch's offset; the camera looks at it
  // Two ground tiles side by side. A: RAMP patches (t in aux[0]). B: TINT patches, at its
  // own origin 8 units east; its NOISE origin makes up the difference, so the field is one
  // field across the seam. 20 field units (metres) to a world unit.
  const OFFB = [OFF[0] + 8, OFF[1], OFF[2]];
  const groundA = new Batch(KIND.PATCH, OFF), groundB = new Batch(KIND.PATCH, OFFB), roofsB = new Batch(KIND.BOX, OFFB);
  const M = 20;
  const pA = Float32Array.from(PARAMS_NEUTRAL), pB = Float32Array.from(PARAMS_NEUTRAL), pRest = Float32Array.from(PARAMS_NEUTRAL);
  pA.set([0.16, 0.30, 0.15, 1], 4); pA.set([0.50, 0.47, 0.44, 1], 8); pA.set([0, 0, M, M], 16);
  pB.set([1.0, 0.80, 0.58, 1], 12); pB.set([8 * M, 0, M, M], 16);
  groundA.params = pA; groundB.params = pB; roofsB.params = pB;
  for (let i = 0; i < 4; i++) for (let j = 0; j < 6; j++) {
    const t = Math.min(1, Math.max(0, (j + 0.5) / 6 + 0.12 * ((i * 7 + j * 3) % 5 - 2) / 2));
    groundA.add([-7 + 2 * i, -5 + 2 * j, -0.02], [2, 2, 0], [t, 0, 0], 0, [255, 0, 255, 255], 0, COLOUR.RAMP);
    const g = 120 + ((i * 5 + j * 11) % 4) * 22;
    groundB.add([-7 + 2 * i, -5 + 2 * j, -0.02], [2, 2, 0], null, 0, [g, g, g, 255], 0, COLOUR.TINT);
  }
  roofsB.add([-3.4, 4.6, 0], [1.2, 0.9, 0.5], null, 0.3, [235, 235, 235, 255], 0, COLOUR.TINT);
  roofsB.add([-1.6, 4.2, 0], [0.9, 1.3, 0.7], null, -0.2, [200, 200, 200, 255], 0, COLOUR.TINT);
  const quads = new Batch(KIND.QUAD, OFF);
  const discs = new Batch(KIND.DISC, OFF);
  const boxes = new Batch(KIND.BOX, OFF);
  const prisms = new Batch(KIND.PRISM, OFF);
  const spheres = new Batch(KIND.SPHERE, OFF);
  const coarse = new Batch(KIND.SPHERE, OFF, true);
  const rods = new Batch(KIND.ROD, OFF);
  const lines = new Batch(KIND.LINE, OFF);
  const glass = new Batch(KIND.SPHERE, OFF);
  const arcs = new Batch(KIND.ARC, OFF);

  // Quads: a row of tiles on the floor, each turned a little more.
  for (let i = 0; i < 5; i++) quads.add([-6 + i * 1.3, -4.6, 0.01], [1.0, 0.6, 0], null, i * 0.3, [230, 180 - i * 25, 60 + i * 30, 255]);
  // Discs.
  for (let i = 0; i < 5; i++) discs.add([1 + i * 1.2, -4.6, 0.01], [0.25 + 0.07 * i, 0, 0], null, 0, [80 + i * 35, 200, 230 - i * 30, 255]);
  // A box and a turned prism.
  boxes.add([5.6, 2.2, 0], [1.6, 1.2, 2.2], null, 0.0, [205, 120, 80, 255]);
  boxes.add([5.8, -0.6, 0], [1.0, 1.0, 1.0], null, 0.6, [120, 170, 110, 255]);
  // VARIANT_ORIENT: a slab leaning 30 degrees toward +x, one record (aux is its local z axis)
  boxes.add([4.2, -2.4, 0], [0.9, 0.3, 1.8], [Math.sin(0.52), 0, Math.cos(0.52)], 0.0, [110, 150, 210, 255], 0, VARIANT_ORIENT);
  prisms.add([-6.0, 2.6, 0], [1.4, 2.2, 1.4], null, -0.5, [150, 130, 200, 255]);

  // A lattice of atoms with bonds.
  const N = [5, 4, 3], a = 1.5, o = [-3.0, -2.0, 0.8];
  const P = (i, j, k) => [o[0] + i * a, o[1] + j * a, o[2] + k * a];
  const cols = [[220, 70, 70, 255], [235, 235, 235, 255], [70, 120, 230, 255]];
  for (let i = 0; i < N[0]; i++) for (let j = 0; j < N[1]; j++) for (let k = 0; k < N[2]; k++) {
    const t = (i + j + k) % 3;
    spheres.add(P(i, j, k), [t === 1 ? 0.28 : 0.42, 0, 0], null, 0, cols[t]);
    if (i + 1 < N[0]) rods.add(P(i, j, k), [0.09, 0, 0], P(i + 1, j, k), 0, [170, 175, 185, 255]);
    if (j + 1 < N[1]) rods.add(P(i, j, k), [0.09, 0, 0], P(i, j + 1, k), 0, [170, 175, 185, 255]);
    if (k + 1 < N[2]) rods.add(P(i, j, k), [0.09, 0, 0], P(i, j, k + 1), 0, [170, 175, 185, 255]);
  }
  // A diagonal rod, to check a general axis, and a vertical one (the basis's other branch).
  rods.add([-6.5, -1.5, 0.2], [0.18, 0, 0], [-4.6, -0.2, 2.6], 0, [240, 200, 60, 255]);
  rods.add([-5.0, -2.6, 0.0], [0.15, 0, 0], [-5.0, -2.6, 1.8], 0, [60, 200, 160, 255]);
  // Coarse (low-poly) spheres, to see the cheaper mesh.
  coarse.add([-6.6, 0.6, 0.5], [0.5, 0, 0], null, 0, [240, 150, 40, 255], 2);
  coarse.add([-5.4, 0.9, 0.5], [0.5, 0, 0], null, 0, [240, 150, 40, 255], 2);

  // The cell's 12 edges as lines.
  const lo = [o[0] - 0.75, o[1] - 0.75, o[2] - 0.75];
  const hi = [o[0] + (N[0] - 1) * a + 0.75, o[1] + (N[1] - 1) * a + 0.75, o[2] + (N[2] - 1) * a + 0.75];
  const C = (b) => [b & 1 ? hi[0] : lo[0], b & 2 ? hi[1] : lo[1], b & 4 ? hi[2] : lo[2]];
  for (let b = 0; b < 8; b++) for (const bit of [1, 2, 4]) if (!(b & bit)) lines.add(C(b), [0.04, 0, 0], C(b | bit), 0, [250, 250, 160, 255]);
  // One fat line across the floor.
  lines.add([-7, 5, 0.2], [0.3, 0, 0], [7, 5.4, 0.2], 0, [255, 90, 160, 255], 0, VARIANT_ROUND);   // round ends, as a tube

  // Arcs: one record each, bent by size[1] (up) and size[2] (across); the last is unbent (its chord).
  arcs.add([-6.5, -3.4, 0.1], [0.08, 2.4, 0], [-1.5, -3.4, 0.1], 0, [255, 200, 90, 255], 0, VARIANT_ROUND);
  arcs.add([-6.5, -3.4, 0.1], [0.06, 1.2, 1.4], [-1.5, -3.4, 0.1], 0, [120, 220, 255, 255], 0, VARIANT_ROUND);
  arcs.add([-6.5, -3.9, 0.1], [0.05, 0, 0], [-1.5, -3.9, 0.1], 0, [200, 200, 200, 255]);
  // Materials on lit marks (variant bits 4..5): matte, metal, glow beside the kind's own.
  spheres.add([2.2, 3.4, 0.55], [0.5, 0, 0], null, 0, [200, 205, 215, 255], 0, MATERIAL.KIND);
  spheres.add([3.4, 3.4, 0.55], [0.5, 0, 0], null, 0, [200, 205, 215, 255], 0, MATERIAL.MATTE);
  spheres.add([4.6, 3.4, 0.55], [0.5, 0, 0], null, 0, [200, 205, 215, 255], 0, MATERIAL.METAL);
  spheres.add([5.8, 3.4, 0.55], [0.5, 0, 0], null, 0, [255, 190, 90, 255], 0, MATERIAL.GLOW);

  // A translucent sphere, last (blending is in the order given).
  glass.add([5.8, -0.6, 1.9], [0.9, 0, 0], null, 0, [120, 200, 255, 110]);

  // The city tile, for ?map=1: canopies, then streets (every casing, then every road), then
  // buildings (a shadow, then a roof in two tones). Painted in that order with no depth test.
  const cDiscs = new Batch(KIND.DISC, OFF), cLines = new Batch(KIND.LINE, OFF), cQuads = new Batch(KIND.QUAD, OFF);
  const pC = Float32Array.from(PARAMS_NEUTRAL);
  pC.set([1.0, 0.86, 0.72, 1], PARAM.TINT); pC.set([0, 0, M, M], PARAM.NOISE);
  for (const b of [cDiscs, cLines, cQuads]) b.params = pC;
  let seed = 12345;
  const rnd = () => ((seed = (seed * 1103515245 + 12345) >>> 0) / 4294967296);
  const streets = [   // x0, y0, x1, y1, width (units; 20 m to a unit), rank
    [-8, 0, 8, 0, 0.6, 2], [-4, -6, -4, 6, 0.35, 1], [3, -6, 3, 6, 0.35, 1],
    [-8, 3, 8, 3, 0.2, 0], [-8, -3.5, 8, -3.5, 0.2, 0], [-0.5, -6, -0.5, 6, 0.2, 0], [6, -6, 6.6, 6, 0.2, 0],
  ];
  // A bent street of 7 segments with ROUND ends (so round joins), and a short cul-de-sac.
  for (let k = 0; k < 7; k++) {
    const a0 = (k / 7) * 1.5, a1 = ((k + 1) / 7) * 1.5;
    streets.push([-7.6 + 3.2 * Math.sin(a0), -5.9 + 2.2 * (1 - Math.cos(a0)), -7.6 + 3.2 * Math.sin(a1), -5.9 + 2.2 * (1 - Math.cos(a1)), 0.3, 0, VARIANT_ROUND]);
  }
  streets.push([-6.6, 1.2, -5.4, 2.2, 0.3, 0, VARIANT_ROUND]);
  const roadCol = [[122, 122, 128, 255], [168, 168, 163, 255], [219, 199, 143, 255]];
  for (const [x0, y0, x1, y1, w, r, v] of streets) cLines.add([x0, y0, 0], [w + 0.16, 0, 0], [x1, y1, 0], 0, [18, 20, 24, 230], 0, VARIANT_CASING | (v || 0));
  for (const rank of [0, 1, 2]) for (const [x0, y0, x1, y1, w, r, v] of streets) if (r === rank) cLines.add([x0, y0, 0], [w, 0, 0], [x1, y1, 0], 0, roadCol[r], 0, v || 0);
  const nearStreet = (x, y, m) => streets.some(([x0, y0, x1, y1, w]) => {
    const dx = x1 - x0, dy = y1 - y0, t = Math.max(0, Math.min(1, ((x - x0) * dx + (y - y0) * dy) / (dx * dx + dy * dy)));
    return Math.hypot(x - x0 - t * dx, y - y0 - t * dy) < w / 2 + m;
  });
  // Mottling patches (the twin's): casing class, NO field. Larger than a tree, so a field on one would show.
  for (let k = 0; k < 14; k++) cDiscs.add([-7.2 + 15 * rnd(), -5.4 + 11 * rnd(), 0], [0.45 + 0.25 * rnd(), 0, 0], null, 0, [70, 96, 58, 150], 0, DISC_CASING);
  for (let gx = -7.5; gx < 8; gx += 0.75) for (let gy = -5.5; gy < 6; gy += 0.62) {
    const x = gx + (rnd() - 0.5) * 0.12, y = gy + (rnd() - 0.5) * 0.1;
    if (nearStreet(x, y, 0.36)) { if (!nearStreet(x, y, 0.2) && rnd() < 0.6) cDiscs.add([x, y, 0], [0.11 + 0.1 * rnd(), 0, 0], null, 0, [28, 51 + Math.round(15 * rnd()), 26, 217], 0, DISC_CASING | DISC_CANOPY_FIELD); continue; }
    if (rnd() < 0.18) { cDiscs.add([x, y, 0], [0.14 + 0.1 * rnd(), 0, 0], null, 0, [28, 51 + Math.round(15 * rnd()), 26, 217], 0, DISC_CASING | DISC_CANOPY_FIELD); continue; }
    const w = 0.4 + 0.18 * rnd(), d = 0.3 + 0.12 * rnd(), g = 150 + Math.round(60 * rnd());
    cQuads.add([x + 0.07, y - 0.07, 0], [w, d, 0], null, 0, [5, 6, 8, 128]);                                   // a cast shadow: literal
    cQuads.add([x, y + d / 4, 0], [w, d / 2, 0], null, 0, [g, g, g, 255], 0, COLOUR.TINT);                      // the lit half of the roof
    cQuads.add([x, y - d / 4, 0], [w, d / 2, 0], null, 0, [g * 0.62, g * 0.62, g * 0.62, 255], 0, COLOUR.TINT); // the dark half
  }
  const rest = [quads, discs, boxes, prisms, spheres, coarse, rods, lines, arcs, glass];
  for (const b of rest) b.params = pRest;
  return { OFF, batches: [groundA, groundB, roofsB, ...rest], city: [groundA, groundB, cDiscs, cLines, cQuads], pA, pB, pRest, pC };
}

// ---- camera (column-major) ----
export function mul(a, b) {
  const o = new Float32Array(16);
  for (let c = 0; c < 4; c++) for (let r = 0; r < 4; r++) {
    let s = 0;
    for (let k = 0; k < 4; k++) s += a[k * 4 + r] * b[c * 4 + k];
    o[c * 4 + r] = s;
  }
  return o;
}
export function lookAt(eye, at, up) {
  const sub = (a, b) => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
  const cross = (a, b) => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
  const dot = (a, b) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
  const norm = (a) => { const l = Math.hypot(...a); return [a[0] / l, a[1] / l, a[2] / l]; };
  const f = norm(sub(at, eye)), s = norm(cross(f, up)), u = cross(s, f);
  return new Float32Array([s[0], u[0], -f[0], 0, s[1], u[1], -f[1], 0, s[2], u[2], -f[2], 0, -dot(s, eye), -dot(u, eye), dot(f, eye), 1]);
}
export function perspective(fovy, aspect, n, f) {
  const t = 1 / Math.tan(fovy / 2);
  return new Float32Array([t / aspect, 0, 0, 0, 0, t, 0, 0, 0, 0, (f + n) / (n - f), -1, 0, 0, (2 * f * n) / (n - f), 0]);
}
export function ortho(h, aspect, n, f) {
  return new Float32Array([1 / (h * aspect), 0, 0, 0, 0, 1 / h, 0, 0, 0, 0, 2 / (n - f), 0, 0, 0, (f + n) / (n - f), 1]);
}

/**
 * The panels and the tile-settings plumbing of the test pages.
 * env: { q (URLSearchParams), canvas, dpr, isMap, state, hit, pA, pB, pRest, pC (from scene()) }.
 * The returned object's `text` is the text provider; the host sets it before the first overlay().
 */
export function createOverlay(env) {
  const { q, canvas, dpr, isMap, state, hit, pA, pB, pRest, pC } = env;
  // ---- the screen overlay: panels, rows, buttons, in device pixels ----
  // A panel is one screen tile: its origin is the batch offset, and its marks are packed per
  // kind in the core's order (quad, line, disc, run), as Scene::frame emits them.
  class Panel {
    constructor() {
      this.o = [0, 0, 0];
      this.quads = new Batch(KIND.QUAD, this.o, false, true);
      this.lines = new Batch(KIND.LINE, this.o, false, true);
      this.discs = new Batch(KIND.DISC, this.o, false, true);
      this.runs = new Batch(KIND.RUN, this.o, false, true);
      this.batches = [this.quads, this.lines, this.discs, this.runs];
    }
    begin(x, y, w, h) {
      this.o[0] = x; this.o[1] = y; this.w = w; this.h = h;
      for (const b of this.batches) b.clear();
      const r = 10 * dpr;
      this.rect(-dpr, -dpr, w + 2 * dpr, h + 2 * dpr, r + dpr, [86, 100, 128, 255]);   // a 1 px (CSS) rim
      this.rect(0, 0, w, h, r, [17, 22, 32, 232]);
    }
    rect(x, y, w, h, r, rgba) { this.quads.add([x + w / 2, y + h / 2, 0], [w, h, 0], [r, 0, 0], 0, rgba); }
    line(x0, y0, x1, y1, w, rgba) { this.lines.add([x0, y0, 0], [w, 0, 0], [x1, y1, 0], 0, rgba); }
    disc(x, y, r, rgba) { this.discs.add([x, y, 0], [r, 0, 0], null, 0, rgba); }
    // align: -1 the ink starts at x, 0 centred on x, 1 the ink ends at x; cy is the row's centre line.
    text(str, style, x, cy, align, rgba) {
      const e = ui.text.place(str, style);
      if (e.w === 0) return e;
      const cx = align < 0 ? x + e.w / 2 - 1 : align > 0 ? x - e.w / 2 + 1 : x;
      this.runs.add([cx, cy, 0], [e.w, e.h, 0], [e.u0, e.v0, e.u1], e.v1, rgba, 0, e.page);
      return e;
    }
  }
  const panels = [new Panel(), new Panel(), new Panel(), new Panel()];

  // ?images=1: a fifth panel of IMAGES (kind 12): a QR-like grid in slot 0, sampled nearest; a
  // translucent gradient in slot 1, sampled linear, tinted; and one image naming slot 7, which
  // holds nothing, so the drawer skips it (stats().missingImages = 1). Each slot is its own
  // batch, as Scene::frame emits them.
  const withImages = q.get('images') === '1';
  const images = [];
  if (withImages) {
    const qr = document.createElement('canvas');
    qr.width = qr.height = 21;
    const g = qr.getContext('2d');
    g.fillStyle = '#fff'; g.fillRect(0, 0, 21, 21);
    g.fillStyle = '#000';
    for (let y = 0; y < 21; y++) for (let x = 0; x < 21; x++) {
      const finder = (cx, cy) => { const dx = Math.abs(x - cx), dy = Math.abs(y - cy), m = Math.max(dx, dy); return m <= 3 && m !== 2; };
      const inFinder = (x < 8 && y < 8) || (x > 12 && y < 8) || (x < 8 && y > 12);
      const on = inFinder ? finder(x < 8 ? 3 : 17, y < 8 ? 3 : 17) : ((x * 7 + y * 13 + x * y) % 5) < 2;
      if (on) g.fillRect(x, y, 1, 1);
    }
    const gr = document.createElement('canvas');
    gr.width = gr.height = 64;
    const h = gr.getContext('2d');
    const lg = h.createLinearGradient(0, 0, 64, 64);
    lg.addColorStop(0, 'rgba(255, 120, 40, 1)'); lg.addColorStop(1, 'rgba(40, 120, 255, 0.25)');
    h.fillStyle = lg; h.fillRect(0, 0, 64, 64);
    images.push({ slot: 0, source: qr, nearest: true }, { slot: 1, source: gr, nearest: false });
    const ip = new Panel();
    ip.imgs = [new Batch(KIND.IMAGE, ip.o, false, true), new Batch(KIND.IMAGE, ip.o, false, true), new Batch(KIND.IMAGE, ip.o, false, true)];
    ip.batches = [ip.quads, ip.imgs[0], ip.imgs[1], ip.imgs[2]];
    ip.image = (b, x, y, w, hh, rgba, slot) => b.add([x + w / 2, y + hh / 2, 0], [w, hh, 0], [0, 0, 1], 1, rgba, 0, slot);
    panels.push(ip);
  }
  // ?icons=1: a panel of the 16 glyph shapes on screen (kind 10, screen): one row at 16 px,
  // two rows at 48 px (the second dark bodies on a light strip), a few turned; and a FLOW
  // line on screen (kind 3, screen, VARIANT_FLOW) at the tile's phase.
  const withIcons = q.get('icons') === '1';
  const pScreenFlow = Float32Array.from(PARAMS_NEUTRAL);
  if (withIcons) {
    pScreenFlow[PARAM.FLOW] = q.has('phase') ? +q.get('phase') : 0.25;
    pScreenFlow[PARAM.FLOW_N] = 8;
    const gp = new Panel();
    gp.glyphs = new Batch(KIND.GLYPH, gp.o, false, true);
    gp.flow = new Batch(KIND.ARC, gp.o, false, true);
    gp.flow.params = pScreenFlow;
    gp.batches = [gp.quads, gp.glyphs, gp.flow, gp.runs];
    panels.push(gp);
  }
  const pPanel = Float32Array.from(PARAMS_NEUTRAL);   // panel 4 shows tile A's ramp and tile B's tint in screen marks
  for (const b of panels[3].batches) b.params = pPanel;
  const INK = [222, 229, 240, 255], DIM = [140, 152, 172, 255], HEAD = [120, 190, 255, 255];

  function overlay(lines, verdict) {
    const W = canvas.width, H = canvas.height, d = dpr;
    if (withImages) {
      const ip = panels[4], side = 168 * d, gap = 12 * d;
      ip.begin(W - 2 * side - 3 * gap - 12 * d, H - side - 2 * gap - 12 * d, 2 * side + 3 * gap, side + 2 * gap);
      for (const b of ip.imgs) b.clear();
      ip.image(ip.imgs[0], gap, gap, side, side, [255, 255, 255, 255], 0);
      ip.image(ip.imgs[1], 2 * gap + side, gap, side, side, [255, 255, 255, 230], 1);
      ip.image(ip.imgs[2], gap, gap, 24 * d, 24 * d, [255, 0, 0, 255], 7);
    }
    if (withIcons) {
      const gp = panels[panels.length - 1], pw2 = 16 * 28 * d + 24 * d, ph2 = 12 * d + 22 * d + 28 * d + 2 * 58 * d + 12 * d;
      gp.begin(W - pw2 - 12 * d, H - ph2 - 12 * d, pw2, ph2);
      gp.glyphs.clear(); gp.flow.clear();
      gp.text('Glyphs 16 px and 48 px', STYLE.SMALL, 12 * d, 12 * d + 7 * d, -1, HEAD);
      gp.flow.add([200 * d, 12 * d + 8 * d, 0], [4 * d, 0, 0], [gp.w - 14 * d, 12 * d + 8 * d, 0], 0, [120, 200, 255, 255], 0, VARIANT_FLOW);
      const y16 = 12 * d + 22 * d + 14 * d;
      const turnedS = { [GLYPH.ARROW]: 0.6, [GLYPH.KEY]: -0.5 };
      for (let k = 0; k < 16; k++) {
        const c = ICON_COLOURS_LIGHT[k % 4];
        gp.glyphs.add([12 * d + 14 * d + k * 28 * d, y16, 0], [8 * d, 0, 0], null, turnedS[k] || 0, [c[0], c[1], c[2], 255], 0, k);
      }
      const y48 = y16 + 14 * d + 29 * d;
      gp.rect(14 * d, y48 + 58 * d - 27 * d, 8 * 56 * d, 54 * d, 6 * d, [226, 230, 236, 255]);   // a light strip for the dark bodies
      for (let k = 0; k < 16; k++) {
        const row = k < 8 ? 0 : 1, c = row === 0 ? ICON_COLOURS_LIGHT[k % 4] : ICON_COLOURS_DARK[k % 4];
        gp.glyphs.add([12 * d + 30 * d + (k % 8) * 56 * d, y48 + row * 58 * d, 0], [24 * d, 0, 0], null, turnedS[k] || 0, [c[0], c[1], c[2], 255], 0, k);
      }
    }
    const narrow = W / d < 700;
    const pad = 12 * d, row = 20 * d, pw = Math.min(330 * d, W - 24 * d);
    const heading = (p, str) => {
      p.text(str, STYLE.HEADING, pad, pad + 8 * d, -1, HEAD);
      p.line(pad, pad + 20 * d, p.w - pad, pad + 20 * d, d, [60, 72, 96, 255]);
    };

    // Panel 1: the engine's own counts, and the verdict.
    const a = panels[0];
    a.begin(12 * d, 12 * d, pw, pad * 2 + 24 * d + lines.length * row + 34 * d);
    heading(a, 'Engine');
    let y = pad + 24 * d + row / 2;
    for (const [k, v] of lines) {
      a.text(k, STYLE.BODY, pad, y, -1, DIM);
      a.text(v, STYLE.BODY, a.w - pad, y, 1, INK);
      y += row;
    }
    const vc = verdict ? verdict.colour : [80, 220, 120, 255];
    a.disc(pad + 8 * d, y + 12 * d, 7 * d, vc);
    a.text(verdict ? verdict.word : 'PASS', STYLE.STATUS, pad + 24 * d, y + 12 * d, -1, vc);
    a.text(verdict ? verdict.note : 'gl error 0', STYLE.SMALL, a.w - pad, y + 12 * d, 1, DIM);

    // Panel 2: strings the platform must shape (joining, reordering, stacked marks, fallback).
    const scripts = [['Arabic', 'مرحبا بالعالم 123'], ['Hindi', 'नमस्ते दुनिया'], ['Thai', 'สวัสดีชาวโลก'], ['Japanese', 'こんにちは世界'], ['Mixed', 'H₂O → água, 25 °C']];
    const b = panels[1];
    const bh = pad * 2 + 24 * d + scripts.length * row;
    if (narrow) b.begin(12 * d, a.o[1] + a.h + 12 * d, pw, bh); else b.begin(W - pw - 12 * d, 12 * d, pw, bh);
    heading(b, 'Scripts, shaped by the platform');
    y = pad + 24 * d + row / 2;
    for (const [k, v] of scripts) {
      b.text(k, STYLE.SMALL, pad, y, -1, DIM);
      b.text(v, STYLE.BODY, b.w - pad, y, 1, INK);
      y += row;
    }

    // Panel 3: a row of rounded buttons and a disc.
    const c = panels[2];
    const names = ['Reset', 'Spin', 'Explore'];
    const bwid = 84 * d, bht = 30 * d, gap = 8 * d;
    const cw = pad * 2 + names.length * bwid + (names.length - 1) * gap + gap + bht;
    c.begin(narrow ? (W - cw) / 2 : 12 * d, H - bht - pad * 2 - 12 * d, cw, bht + pad * 2);
    let x = pad;
    names.forEach((n, i) => {
      c.rect(x, pad, bwid, bht, 7 * d, i === 2 ? [38, 128, 96, 255] : [48, 62, 90, 255]);
      c.text(n, STYLE.BUTTON, x + bwid / 2, pad + bht / 2, 0, INK);
      x += bwid + gap;
    });
    c.disc(x + bht / 2, pad + bht / 2, bht / 2, [235, 180, 60, 255]);
    c.disc(x + bht / 2, pad + bht / 2, bht / 2 - 4 * d, [17, 22, 32, 255]);

    // Panel 4: the tile settings. A slider for DIM (drag it), the field's switch, and screen
    // discs in RAMP and TINT mode reading the same settings the ground does.
    const s4 = panels[3];
    const sh = pad * 2 + 24 * d + 3 * row + 10 * d;
    if (narrow) s4.begin(12 * d, c.o[1] - sh - 12 * d, pw, sh); else s4.begin(W - pw - 12 * d, b.o[1] + b.h + 12 * d, pw, sh);
    heading(s4, 'Tile settings');
    y = pad + 24 * d + row / 2;
    s4.text('night dim', STYLE.BODY, pad, y, -1, DIM);
    s4.text(state.dim.toFixed(2), STYLE.BODY, s4.w - pad, y, 1, INK);
    const tx0 = pad + 96 * d, tx1 = s4.w - pad - 48 * d, f = (state.dim - 0.3) / 0.7, kx = tx0 + f * (tx1 - tx0);
    s4.rect(tx0, y - 2 * d, tx1 - tx0, 4 * d, 2 * d, [52, 64, 90, 255]);
    s4.rect(tx0, y - 2 * d, kx - tx0, 4 * d, 2 * d, [120, 190, 255, 255]);
    s4.disc(kx, y, 7 * d, [222, 229, 240, 255]);
    hit.slider = [s4.o[0] + tx0, s4.o[1] + y - 12 * d, s4.o[0] + tx1, s4.o[1] + y + 12 * d];
    y += row + 4 * d;
    s4.text('ground field', STYLE.BODY, pad, y, -1, DIM);
    const bw4 = 56 * d, bx4 = s4.w - pad - bw4;
    s4.rect(bx4, y - 11 * d, bw4, 22 * d, 6 * d, state.tex > 0 ? [38, 128, 96, 255] : [70, 50, 56, 255]);
    s4.text(state.tex > 0 ? 'on' : 'off', STYLE.BUTTON, bx4 + bw4 / 2, y, 0, INK);
    hit.button = [s4.o[0] + bx4, s4.o[1] + y - 11 * d, s4.o[0] + bx4 + bw4, s4.o[1] + y + 11 * d];
    y += row + 4 * d;
    s4.text('ramp', STYLE.SMALL, pad, y, -1, DIM);
    for (let i = 0; i < 5; i++) s4.discs.add([pad + (48 + 22 * i) * d, y, 0], [8 * d, 0, 0], null, i / 4, [255, 0, 255, 255], 0, COLOUR.RAMP);   // a screen mark's t is its angle
    s4.text('tint', STYLE.SMALL, s4.w - pad - 60 * d, y, 1, DIM);
    for (let i = 0; i < 2; i++) s4.discs.add([s4.w - pad - (8 + 22 * (1 - i)) * d, y, 0], [8 * d, 0, 0], null, 0, [150 + 80 * i, 150 + 80 * i, 150 + 80 * i, 255], 0, COLOUR.TINT);
  }

  // The settings, written into the tiles' 32 numbers: no instance changes.
  function applyState() {
    for (const p of [pA, pB, pRest, pPanel, pC]) { p[0] = state.dim; p[1] = state.dim; p[2] = state.dim; }
    pA[PARAM.TEX] = state.tex; pB[PARAM.TEX] = state.tex; pC[PARAM.TEX] = state.tex;
    if (q.has('fade')) for (const p of [pA, pB, pRest, pC]) p[PARAM.FADE] = +q.get('fade');
    if (isMap) {
      pC[PARAM.FLOOR] = q.has('floor') ? +q.get('floor') * dpr : 0.5 * dpr;
      const al = (q.get('alphas') || '1,1,1,1').split(',').map(Number);
      pC[PARAM.FLOOR_CASING] = +(q.get('floorc') || 0) * dpr;
      pA[PARAM.A_GROUND] = pB[PARAM.A_GROUND] = al[0]; pC[PARAM.A_CASING] = al[1]; pC[PARAM.A_ROAD] = al[2]; pC[PARAM.A_ROOF] = al[3];
      if (q.get('ring') === '1') for (const p of [pA, pB, pC]) p.set([canvas.width / 2, canvas.height / 2, 0.46 * Math.min(canvas.width, canvas.height)], PARAM.RING);
    } else if (q.get('ring') === '1') {
      // the 3D scene: a small ring on the lit marks only (not the ground), so the shadow pass's
      // ring shows: what the picture fades out casts nothing
      pRest.set([canvas.width / 2, canvas.height / 2, 0.22 * Math.min(canvas.width, canvas.height)], PARAM.RING);
    }
    pPanel.set(pA.subarray(4, 12), 4); pPanel.set(pB.subarray(12, 16), 12);
  }

  // the host's textures, given to the drawer once it exists
  const installImages = (drawer) => { for (const im of images) drawer.setImage(im.slot, im.source, { nearest: im.nearest }); };
  const ui = { panels, pPanel, overlay, applyState, installImages, imageSlots: images.map((im) => im.slot), text: env.text || null };
  return ui;
}

// The view-projection of a frame (column-major): the orbit camera, or the flat map's top-down view.
export function viewProjection(q, isMap, OFF, canvas, dpr, az, el, dist) {
  const at = [OFF[0], OFF[1], OFF[2] + 1.2];
  const eye = [at[0] + dist * Math.cos(el) * Math.sin(az), at[1] - dist * Math.cos(el) * Math.cos(az), at[2] + dist * Math.sin(el)];
  const aspect = canvas.width / canvas.height;
  const fov = (38 * Math.PI) / 180 * (aspect < 1 ? 1.9 : 1);
  const proj = q.get('ortho') === '1' ? ortho(6.2, aspect, 0.1, 100) : perspective(fov, aspect, 0.1, 100);
  // The flat map: top-down, x east, y north, mapPpu device pixels to a world unit.
  const mapPpu = +(q.get('ppu') || 70) * dpr;
  const sx = 2 * mapPpu / canvas.width, sy = 2 * mapPpu / canvas.height;
  const vp = isMap ? new Float32Array([sx, 0, 0, 0, 0, sy, 0, 0, 0, 0, -0.01, 0, -sx * OFF[0], -sy * OFF[1], 0, 1]) : mul(proj, lookAt(eye, at, [0, 0, 1]));
  return { vp, mapPpu };
}

// ---- ?resident=1: the world batches drawn from resident buffers ----
// Puts every world batch of `all` (in place, by index) into two resident buffers, A (the
// first two world batches) and B (the rest), each batch a range of its buffer, so the
// picture is the same scene drawn with no record uploaded. On the way it exercises the API:
// A is made from one record, grown by update() to its records and three junk records (a
// reallocation), then shrunk by update() to its records alone (in place, the allocation
// kept); a junk handle is released and must be the number resident() hands B; and draw()
// must throw on a range past A's records (after the shrink), on the released handle, and on
// a resident image batch with no `slot`. `tryDraw(batches)` draws and returns the error's
// message, or null. Returns what the page reports in document.body.dataset.resident.
export function makeResident(drawer, all, tryDraw) {
  const idx = [];
  for (let i = 0; i < all.length; i++) if (!all[i].screen && all[i].count > 0) idx.push(i);
  const cat = (list, extra) => {
    let n = extra * INSTANCE_BYTES;
    for (const i of list) n += all[i].count * INSTANCE_BYTES;
    const out = new Uint8Array(n), firsts = [];
    let at = 0;
    for (const i of list) { firsts.push(at / INSTANCE_BYTES); out.set(all[i].bytes.subarray(0, all[i].count * INSTANCE_BYTES), at); at += all[i].count * INSTANCE_BYTES; }
    for (let k = 0; k < extra * INSTANCE_BYTES; k++) out[at + k] = (k * 37) & 255;   // junk: never drawn
    return { out, firsts };
  };
  const iA = idx.slice(0, 2), iB = idx.slice(2);
  const A = cat(iA, 0), Agrown = cat(iA, 3), B = cat(iB, 0);
  const st = () => ({ buffers: drawer.stats().residentBuffers, bytes: drawer.stats().residentBytes });
  const log = {};
  const hA = drawer.resident(A.out.subarray(0, INSTANCE_BYTES));
  log.afterCreate = st();
  drawer.update(hA, Agrown.out);
  log.afterGrow = st();
  drawer.update(hA, A.out);
  log.afterShrink = st();
  const nA = A.out.byteLength / INSTANCE_BYTES;
  log.pastRange = tryDraw([{ kind: all[iA[0]].kind, handle: hA, first: nA, count: 1 }]);   // the junk records, now past the end
  const hJ = drawer.resident(cat(iA.slice(0, 1), 2).out);
  drawer.release(hJ);
  log.afterRelease = st();
  log.released = tryDraw([{ kind: all[iA[0]].kind, handle: hJ, first: 0, count: 1 }]);
  log.imageNoSlot = tryDraw([{ kind: KIND.IMAGE, screen: true, handle: hA, first: 0, count: 1 }]);
  const hB = drawer.resident(B.out);
  log.reused = hB === hJ;
  log.handles = [hA, hJ, hB];
  log.final = st();
  const swap = (list, h, firsts) => list.forEach((i, j) => {
    const b = all[i];
    all[i] = { kind: b.kind, handle: h, first: firsts[j], count: b.count, offset: b.offset, params: b.params, coarse: b.coarse, screen: false };
  });
  swap(iA, hA, A.firsts);
  swap(iB, hB, B.firsts);
  log.worldBytes = A.out.byteLength + B.out.byteLength;
  log.imageBytes = 0;
  log.ok = log.reused && !!log.pastRange && !!log.released && !!log.imageNoSlot
    && log.afterGrow.bytes === Agrown.out.byteLength && log.afterShrink.bytes === Agrown.out.byteLength
    && log.final.buffers === 2 && log.final.bytes === Agrown.out.byteLength + B.out.byteLength;
  return log;
}

// ?resident=1&images=1: the image panel's three image batches (laid out once, by the first
// overlay()) moved into one resident buffer, each with its `slot` (the slot its records
// name, which the host knows; the drawer cannot read it from a resident buffer). log.imageBytes
// is what the bytes page uploads for them: the batches whose slot holds a texture (`filled`);
// the one naming an empty slot is skipped there without an upload.
export function makeResidentImages(drawer, all, ip, log, filled) {
  let n = 0;
  for (const b of ip.imgs) n += b.count * INSTANCE_BYTES;
  const out = new Uint8Array(n), firsts = [];
  let at = 0;
  for (const b of ip.imgs) { firsts.push(at / INSTANCE_BYTES); out.set(b.bytes, at); at += b.bytes.byteLength; }
  const h = drawer.resident(out);
  ip.imgs.forEach((b, j) => {
    const i = all.indexOf(b);
    all[i] = { kind: KIND.IMAGE, screen: true, handle: h, first: firsts[j], count: b.count, offset: b.offset, params: b.params, slot: b.bytes[51] };
  });
  log.imageBytes = 0;
  for (const b of ip.imgs) if (filled.includes(b.bytes[51])) log.imageBytes += b.bytes.byteLength;
  log.imageResidentBytes = n;
  log.imageHandle = h;
}

// ---- ?icons=1: the glyph shapes, FLOW on links, and a huge ground patch under the RING ----
// World: the 16 shapes in two rows (light bodies, half size 0.33; dark bodies, half size 0.22;
// a few turned), a translucent one, and a sphere behind the DOCUMENT's transparent margin
// (it must show: a glyph writes no depth where it covers under half). They stand on ONE ground
// patch 400 units wide whose far corners lie behind the eye, with the RING on: it must be
// drawn (until 2026-11-22 the ring was taken per vertex and such a patch vanished). FLOW links
// at a fixed phase (`?phase=`, default 0.25, so the pictures are deterministic): a straight
// tube, a thin round-ended one, a bent arc, a round-ended arc bent across, and one arc drawn
// from a RESIDENT buffer (iconsResident). `ribbons` are drawn by a draw() of their own with
// flatLines: a flat ribbon line and a flat round-ended arc. The camera: az 0, el 18.
export const ICON_COLOURS_LIGHT = [[236, 238, 242], [250, 214, 120], [170, 226, 180], [160, 205, 250]];
export const ICON_COLOURS_DARK = [[40, 46, 64], [120, 40, 52], [26, 82, 60], [36, 64, 128]];
export function iconsScene(q, OFF, canvas) {
  const phase = q.has('phase') ? +q.get('phase') : 0.25;
  const pG = Float32Array.from(PARAMS_NEUTRAL), pI = Float32Array.from(PARAMS_NEUTRAL), pF = Float32Array.from(PARAMS_NEUTRAL);
  pF[PARAM.FLOW] = phase; pF[PARAM.FLOW_N] = 0;   // 0: six dashes
  const pF8 = Float32Array.from(pF); pF8[PARAM.FLOW_N] = 9;
  const ground = new Batch(KIND.PATCH, OFF); ground.params = pG;
  ground.add([0, 0, -0.02], [400, 400, 0], null, 0, [66, 104, 74, 255]);
  pG.set([canvas.width / 2, canvas.height / 2, 0.46 * Math.min(canvas.width, canvas.height)], PARAM.RING);
  const glyphs = new Batch(KIND.GLYPH, OFF); glyphs.params = pI;
  const turned = { [GLYPH.ARROW]: 0.6, [GLYPH.KEY]: -0.5, [GLYPH.DOCUMENT]: 0.25, [GLYPH.CHAT]: -0.2 };
  for (let k = 0; k < 16; k++) {
    const x = -5.625 + 0.75 * k;
    const cl = ICON_COLOURS_LIGHT[k % 4], cd = ICON_COLOURS_DARK[k % 4];
    glyphs.add([x, 0, 1.15], [0.33, 0, 0], null, turned[k] || 0, [cl[0], cl[1], cl[2], 255], 0, k);
    glyphs.add([x, 0, 0.3], [0.22, 0, 0], null, k % 3 === 0 ? -(turned[k] || 0.4) : 0, [cd[0], cd[1], cd[2], 255], 0, k);
  }
  // one translucent glyph beside the rows, to see blending
  glyphs.add([-6.7, -0.8, 0.9], [0.5, 0, 0], null, 0, [255, 255, 255, 110], 0, GLYPH.PHOTO);
  // a lit sphere behind the DOCUMENT's transparent right margin: the margin must not hide it
  const behind = new Batch(KIND.SPHERE, OFF); behind.params = pI;
  behind.add([-1.125 + 0.27, 0.8, 1.15], [0.15, 0, 0], null, 0, [230, 90, 60, 255]);
  const lines = new Batch(KIND.LINE, OFF); lines.params = pF;
  lines.add([-7.0, -3, 0.3], [0.14, 0, 0], [-3.6, -3, 0.3], 0, [90, 200, 255, 255], 0, VARIANT_FLOW);
  lines.add([-7.0, -3, 0.8], [0.06, 0, 0], [-3.6, -3, 0.8], 0, [255, 180, 80, 255], 0, VARIANT_FLOW | VARIANT_ROUND);
  const arcs = new Batch(KIND.ARC, OFF); arcs.params = pF8;
  arcs.add([-2.6, -3, 0.15], [0.12, 1.2, 0], [2.6, -3, 0.15], 0, [255, 120, 170, 255], 0, VARIANT_FLOW);
  arcs.add([-2.6, -3, 0.15], [0.09, 0.4, 1.0], [2.6, -3, 0.15], 0, [140, 255, 160, 255], 0, VARIANT_FLOW | VARIANT_ROUND);
  // the resident one: its records go into a buffer (iconsResident); the batch is swapped then
  const resArc = new Batch(KIND.ARC, OFF); resArc.params = pF;
  resArc.add([3.4, -3, 0.15], [0.12, 1.2, 0], [7.0, -3, 0.15], 0, [255, 230, 90, 255], 0, VARIANT_FLOW | VARIANT_ROUND);
  // flat ribbons (a second draw() with flatLines): a line and a round-ended arc on the ground
  const rLines = new Batch(KIND.LINE, OFF); rLines.params = pF;
  rLines.add([-6.8, -5.2, 0.02], [0.3, 0, 0], [-3.6, -5.2, 0.02], 0, [255, 255, 255, 255], 0, VARIANT_FLOW);
  const rArcs = new Batch(KIND.ARC, OFF); rArcs.params = pF8;
  rArcs.add([-2.8, -5.0, 0.02], [0.3, 0, 0.9], [0.8, -5.0, 0.02], 0, [255, 210, 120, 255], 0, VARIANT_FLOW | VARIANT_ROUND);
  return { phase, pG, pI, pF, pF8, world: [ground, behind, glyphs, lines, arcs, resArc], ribbons: [rLines, rArcs], resArc };
}

// Put the resident arc's records in a buffer and swap its batch in `all` for one that names
// it. Then draw it alone at two phases: the dashes move and no record is uploaded
// (bytesUploaded 0 both times; paramUploads counts the changed setting). `drawOne(batches)`
// draws and returns a copy of stats(). Returns what the page reports in dataset.flow.
export function iconsResident(drawer, all, ic, drawOne) {
  const h = drawer.resident(ic.resArc.bytes);
  const i = all.indexOf(ic.resArc);
  const rb = { kind: KIND.ARC, handle: h, first: 0, count: 1, offset: ic.resArc.offset, params: ic.pF };
  all[i] = rb;
  const p2 = Float32Array.from(ic.pF);
  const a = drawOne([rb]);
  const was = ic.pF[PARAM.FLOW];
  rb.params = p2; p2[PARAM.FLOW] = was + 0.37;
  const b = drawOne([rb]);
  p2[PARAM.FLOW] = was + 1000.5;   // a large phase: only fract() matters
  const c = drawOne([rb]);
  rb.params = ic.pF;
  const log = { handle: h, bytesUploaded: [a.bytesUploaded, b.bytesUploaded, c.bytesUploaded], paramUploads: [a.paramUploads, b.paramUploads, c.paramUploads], drawCalls: [a.drawCalls, b.drawCalls, c.drawCalls] };
  log.ok = log.bytesUploaded.every((x) => x === 0) && log.paramUploads[1] >= 1 && log.paramUploads[2] >= 1 && log.drawCalls.every((x) => x === 1);
  return log;
}

// The counts of two draw() calls of one frame, added (a copy; stats() is one object).
export function sumStats(a, b) {
  const o = { ...b };
  for (const k of ['drawCalls', 'instances', 'bytesUploaded', 'paramUploads', 'uniformBytes', 'missingImages']) if (typeof b[k] === 'number') o[k] = (a[k] || 0) + b[k];
  return o;
}

// ---- ?tiles=1, ?tiles=2: many small tiles, each with its own matrix (BATCH MATRIX in draw.js) ----
// tiles=1 is the twin's street tier in small: a flat map (no depth test) of `n` tiles (default
// 48, an 8 x 6 grid; ?n=N for the timing page), each in its own local frame (0..10 units) with its
// own affine to the screen (a scale of 9 to 9.75 device pixels a unit and a turn of up to 0.08
// radians), its own pixelsPerUnit and its own tile settings (DIM, RAMP, TINT, NOISE, the field,
// FLOOR, the RING about the canvas's centre): a RAMP ground patch, three trees, two casings and
// two streets, two TINT roofs and a cast shadow, in five batches. tiles=2 is a perspective view
// of 42 tiles (7 x 6) on one plane, each tile's matrix the camera's times its own translation,
// with the depth test on and lit marks (a METAL box, a sphere, a rod, a tube) and two
// camera-facing glyphs a tile, so the eye and the billboards are taken from each batch's own
// matrix. A tile is { batches, vp, ppu }; opts is the draw() options every tile shares.
export function tilesScene(q, canvas) {
  const W = canvas.width, H = canvas.height;
  const three = q.get('tiles') === '2';
  const n = Math.max(1, +(q.get('n') || (three ? 42 : 48)) | 0);
  const tiles = [];
  if (!three) {
    const cols = Math.max(1, Math.round(Math.sqrt(n * W / H)));
    const rows = Math.ceil(n / cols);
    const cw = W / cols, ch = H / rows;
    for (let i = 0; i < n; i++) {
      const cx = i % cols, ry = Math.floor(i / cols);
      const s = (9 + 0.25 * (i % 4)) * Math.min(cw / 120, ch / 100), th = 0.04 * ((i % 5) - 2);
      const ox = cx * cw + 0.08 * cw, oy = ry * ch + 0.05 * ch + (th < 0 ? 10 * s * Math.sin(-th) : 0);
      const c = Math.cos(th), sn = Math.sin(th);
      const vp = new Float32Array([2 * s * c / W, 2 * s * sn / H, 0, 0, -2 * s * sn / W, 2 * s * c / H, 0, 0, 0, 0, -0.01, 0, 2 * ox / W - 1, 2 * oy / H - 1, 0, 1]);
      const p = Float32Array.from(PARAMS_NEUTRAL);
      const d = 0.7 + 0.3 * ((i * 7) % 5) / 4;
      p.set([d, d, d, 1], PARAM.DIM);
      p.set([0.14 + 0.02 * (i % 3), 0.30, 0.13], PARAM.RAMP0); p.set([0.52, 0.48 - 0.03 * (i % 4), 0.42], PARAM.RAMP1);
      p.set([1.0, 0.72 + 0.05 * (i % 3), 0.55], PARAM.TINT);
      p.set([cx * 200, ry * 200, 20, 20], PARAM.NOISE); p[PARAM.TEX] = 1;
      p[PARAM.FLOOR] = 0.6; p[PARAM.FLOOR_CASING] = 1.1;
      p.set([W / 2, H / 2, 0.62 * Math.min(W, H)], PARAM.RING);
      const ground = new Batch(KIND.PATCH), trees = new Batch(KIND.DISC), casings = new Batch(KIND.LINE), streets = new Batch(KIND.LINE), roofs = new Batch(KIND.QUAD);
      ground.add([5, 5, 0], [10, 10, 0], [0.25 + 0.5 * ((i * 3) % 7) / 6, 0, 0], 0, [255, 255, 255, 255], 0, COLOUR.RAMP);
      for (let k = 0; k < 3; k++) trees.add([1.6 + 3.2 * k, 8.3 - 0.6 * (k % 2), 0], [0.7 + 0.15 * ((i + k) % 3), 0, 0], null, 0, [46, 92, 50, 235], 0, DISC_CASING | DISC_CANOPY_FIELD);
      casings.add([0.5, 3, 0], [1.1, 0, 0], [9.5, 3.4, 0], 0, [70, 70, 74, 255], 0, VARIANT_CASING | VARIANT_ROUND);
      casings.add([6.5, 0.5, 0], [0.9, 0, 0], [6.1, 9.5, 0], 0, [70, 70, 74, 255], 0, VARIANT_CASING | VARIANT_ROUND);
      streets.add([0.5, 3, 0], [0.7, 0, 0], [9.5, 3.4, 0], 0, [236, 232, 220, 255], 0, VARIANT_ROUND);
      streets.add([6.5, 0.5, 0], [0.5, 0, 0], [6.1, 9.5, 0], 0, [236, 232, 220, 255], 0, VARIANT_ROUND);
      roofs.add([2.9, 5.7, 0], [2.4, 1.6, 0], null, 0.12, [0, 0, 0, 80]);   // a cast shadow (a literal quad)
      roofs.add([2.6, 6.0, 0], [2.4, 1.6, 0], null, 0.12, [200, 128, 96, 255], 0, COLOUR.TINT);
      roofs.add([8.3, 1.6, 0], [1.6, 1.8, 0], null, -0.3, [180, 170, 160, 255], 0, COLOUR.TINT);
      const batches = [ground, trees, casings, streets, roofs];
      for (const b of batches) b.params = p;
      tiles.push({ batches, vp, ppu: s });
    }
    return { tiles, opts: { viewport: [0, 0, W, H], depthTest: false }, flat: true };
  }
  const aspect = W / H;
  const camVP = mul(perspective((40 * Math.PI) / 180, aspect, 0.5, 200), lookAt([4, -34, 22], [0, 1, 0], [0, 0, 1]));
  const cols = 7;
  for (let i = 0; i < n; i++) {
    const cx = i % cols, ry = Math.floor(i / cols);
    const tx = (cx - cols / 2) * 5, ty = (ry - 3) * 5;
    const T = new Float32Array([1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, tx, ty, 0, 1]);
    const vp = mul(camVP, T);
    const p = Float32Array.from(PARAMS_NEUTRAL);
    const d = 0.75 + 0.25 * ((i * 3) % 4) / 3;
    p.set([d, d, d, 1], PARAM.DIM);
    p.set([0.20, 0.34, 0.18], PARAM.RAMP0); p.set([0.46, 0.44, 0.40], PARAM.RAMP1);
    const ground = new Batch(KIND.PATCH), boxes = new Batch(KIND.BOX), spheres = new Batch(KIND.SPHERE), rods = new Batch(KIND.ROD), tubes = new Batch(KIND.LINE), glyphs = new Batch(KIND.GLYPH);
    ground.add([2, 2, 0], [4.6, 4.6, 0], [((i * 5) % 7) / 6, 0, 0], 0, [255, 255, 255, 255], 0, COLOUR.RAMP);
    boxes.add([1.2, 1.4, 0], [1.2, 1.0, 0.8 + 0.3 * (i % 4)], null, 0.2 * (i % 3), [205, 130 + 10 * (i % 5), 90, 255], 0, MATERIAL.METAL);
    spheres.add([3.0, 2.8, 0.6], [0.55, 0, 0], null, 0, [120, 170, 230, 255]);
    rods.add([3.2, 0.8, 0.2], [0.15, 0, 0], [3.6, 1.2, 1.6], 0, [230, 200, 120, 255]);
    tubes.add([0.4, 3.6, 0.15], [0.12, 0, 0], [2.4, 3.8, 0.15], 0, [240, 240, 240, 255]);
    const cl = ICON_COLOURS_LIGHT[i % 4], cd = ICON_COLOURS_DARK[i % 4];
    glyphs.add([1.2, 1.4, 2.6], [0.45, 0, 0], null, 0.3 * (i % 3), [cl[0], cl[1], cl[2], 255], 0, 1 + (i % 15));
    glyphs.add([3.0, 2.8, 1.7], [0.3, 0, 0], null, 0, [cd[0], cd[1], cd[2], 255], 0, 1 + ((i + 7) % 15));
    const batches = [ground, boxes, spheres, rods, tubes, glyphs];
    for (const b of batches) b.params = p;
    tiles.push({ batches, vp, ppu: 0 });
  }
  return { tiles, opts: { viewport: [0, 0, W, H], light: [0.35, -0.5, 0.8], ambient: 0.25 }, flat: false };
}

// A batch as draw() takes it, with every field the engine reads (and `extra` on top).
const asBatch = (b, extra) => Object.assign({ kind: b.kind, count: b.count, bytes: b.bytes, handle: b.handle, first: b.first, offset: b.offset, params: b.params, coarse: b.coarse, screen: b.screen, shade: b.shade, blend: b.blend }, extra);
// The batches of `list` (bytes) in ONE resident buffer, each a range of it; returns the new batches.
export function toResident(drawer, list) {
  let n = 0;
  for (const b of list) n += b.count * INSTANCE_BYTES;
  const out = new Uint8Array(n), firsts = [];
  let at = 0;
  for (const b of list) { firsts.push(at / INSTANCE_BYTES); out.set(b.bytes.subarray(0, b.count * INSTANCE_BYTES), at); at += b.count * INSTANCE_BYTES; }
  const h = drawer.resident(out);
  return list.map((b, j) => { const r = asBatch(b, { handle: h, first: firsts[j] }); delete r.bytes; return r; });
}
// ?resident=1: every tile's batches in a buffer of its own (the twin's way); returns the bytes made resident.
export function tilesResident(drawer, ts) {
  let bytes = 0;
  for (const t of ts.tiles) { for (const b of t.batches) bytes += b.count * INSTANCE_BYTES; t.batches = toResident(drawer, t.batches); }
  return bytes;
}
// One frame of the tiles: PER CALL (one draw() per tile with its matrix as the call's vp and its
// pixelsPerUnit in opts, then the state the twin puts back after each, `restore()`; the panels in
// a last draw()) or in ONE draw() (each batch carrying its tile's `vp` and `pixelsPerUnit`, the
// panels after them). draw(batches, vp, opts) draws and returns a copy of stats(). Returns the
// counts summed over the frame's calls.
export function tilesFrame(ts, panels, perCall, draw, restore) {
  const parts = [];
  if (perCall) {
    for (const t of ts.tiles) {
      parts.push(draw(t.batches, t.vp, t.ppu > 0 ? { ...ts.opts, pixelsPerUnit: t.ppu } : ts.opts));
      if (restore) restore();
    }
    if (panels.length) parts.push(draw(panels, IDENTITY, { viewport: ts.opts.viewport }));
  } else {
    if (!ts.one) {
      ts.one = [];
      for (const t of ts.tiles) for (const b of t.batches) ts.one.push(asBatch(b, t.ppu > 0 ? { vp: t.vp, pixelsPerUnit: t.ppu } : { vp: t.vp }));
    }
    parts.push(draw(ts.one.concat(panels), null, ts.opts));
  }
  let s = parts[0];
  for (let k = 1; k < parts.length; k++) s = sumStats(s, parts[k]);
  return { ...s, calls: parts.length };
}
export const IDENTITY = new Float32Array([1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]);

// ---- ?add=1: BLEND 'add' (draw.js), checked against the arithmetic ----
// World units are device pixels here (x right, y UP from the canvas's bottom; vp maps them to
// clip). Three draw() calls a frame:
//   main   (no depth test) a dark ground on the left half and a bright one on the right; on each,
//          two overlapping ADDITIVE world discs (A: rgb 200 40 20 at alpha 128; B: 30 120 200 at
//          255), and the same two as additive SCREEN discs lower down. The last batch adds.
//   depthA (depth test on) one additive world disc at z 0.5, over the dark ground
//   depthB (depth test on) an opaque quad at z 0.2 under it (farther from this top-down view):
//          an additive batch writes no depth, so the quad must show where the disc is.
// `points` are where the screenshot is checked, top-left device pixels, with the colour the
// arithmetic gives (8-bit: dst + round(src * alpha), clamped at 255, one add after another).
export function addScene(canvas) {
  const W = canvas.width, H = canvas.height;
  const vp = new Float32Array([2 / W, 0, 0, 0, 0, 2 / H, 0, 0, 0, 0, -0.01, 0, -1, -1, 0, 1]);
  const DARK = [20, 22, 30], BRIGHT = [210, 200, 190], A = [200, 40, 20, 128], B = [30, 120, 200, 255], Q = [90, 160, 110];
  const ground = new Batch(KIND.QUAD);
  ground.add([W / 4, H / 2, 0], [W / 2, H, 0], null, 0, [...DARK, 255]);
  ground.add([3 * W / 4, H / 2, 0], [W / 2, H, 0], null, 0, [...BRIGHT, 255]);
  const r = Math.round(Math.min(W, H) * 0.11), dx = Math.round(r * 0.6);
  const wy = Math.round(H * 0.75), sy = Math.round(H * 0.55);   // world discs' y (from the bottom), screen discs' y (from the top)
  const world = new Batch(KIND.DISC), scr = new Batch(KIND.DISC, [0, 0, 0], false, true);
  for (const cx of [Math.round(W / 4), Math.round(3 * W / 4)]) {
    world.add([cx - dx, wy, 0], [r, 0, 0], null, 0, A);
    world.add([cx + dx, wy, 0], [r, 0, 0], null, 0, B);
    scr.add([cx - dx, sy, 0], [r, 0, 0], null, 0, A);
    scr.add([cx + dx, sy, 0], [r, 0, 0], null, 0, B);
  }
  world.blend = 'add'; scr.blend = 'add';
  const dz = new Batch(KIND.DISC), dq = new Batch(KIND.QUAD);
  const qx = Math.round(W / 4), qy = Math.round(H * 0.12);
  dz.add([qx, qy, 0.5], [r * 0.5, 0, 0], null, 0, [255, 255, 255, 255]); dz.blend = 'add';
  dq.add([qx, qy, 0.2], [r * 1.6, r * 0.8, 0], null, 0, [...Q, 255]);
  const add = (d, c) => d.map((v, k) => Math.min(255, v + Math.round(c[k] * c[3] / 255)));
  const points = [];
  for (const [g, cx, name] of [[DARK, Math.round(W / 4), 'dark'], [BRIGHT, Math.round(3 * W / 4), 'bright']]) {
    for (const [y, what] of [[H - wy, 'world'], [sy, 'screen']]) {
      points.push({ what: what + ' ' + name + ' ground', x: cx, y: y - r - Math.round(r * 0.5), want: g });
      points.push({ what: what + ' ' + name + ' A alone', x: cx - dx - Math.round(r * 0.7), y, want: add(g, A) });
      points.push({ what: what + ' ' + name + ' B alone', x: cx + dx + Math.round(r * 0.7), y, want: add(g, B) });
      points.push({ what: what + ' ' + name + ' A then B', x: cx, y, want: add(add(g, A), B) });
    }
  }
  points.push({ what: 'depth: the quad over an additive disc nearer the eye', x: qx, y: H - qy, want: Q });
  return { vp, main: [ground, world, scr], depthA: [dz], depthB: [dq], points, opts: { viewport: [0, 0, W, H], depthTest: false } };
}
// The GL blend and depth-write state now, as names (draw.js's GL STATE promises it on return).
export function glBlendState(gl) {
  const nm = (v) => ({ [gl.ONE]: 'ONE', [gl.ZERO]: 'ZERO', [gl.SRC_ALPHA]: 'SRC_ALPHA', [gl.ONE_MINUS_SRC_ALPHA]: 'ONE_MINUS_SRC_ALPHA', [gl.FUNC_ADD]: 'FUNC_ADD' }[v] || String(v));
  return { blend: gl.isEnabled(gl.BLEND), srcRGB: nm(gl.getParameter(gl.BLEND_SRC_RGB)), dstRGB: nm(gl.getParameter(gl.BLEND_DST_RGB)),
    srcAlpha: nm(gl.getParameter(gl.BLEND_SRC_ALPHA)), dstAlpha: nm(gl.getParameter(gl.BLEND_DST_ALPHA)),
    equation: nm(gl.getParameter(gl.BLEND_EQUATION_RGB)), depthMask: gl.getParameter(gl.DEPTH_WRITEMASK) };
}
export const BLEND_PROMISED = { blend: true, srcRGB: 'SRC_ALPHA', dstRGB: 'ONE_MINUS_SRC_ALPHA', srcAlpha: 'ONE', dstAlpha: 'ONE_MINUS_SRC_ALPHA', equation: 'FUNC_ADD', depthMask: true };

// ?tiles=1|2: what draw() must refuse (BATCH MATRIX, BLEND), each tried once before the first
// frame. tryDraw(batches, vp, opts) draws and returns the error's message, or null if none.
// The last try is a draw that must NOT throw: a batch's own pixelsPerUnit with shadows.
export function tilesRefusals(ts, tryDraw) {
  const b = ts.tiles[0].batches[0], vp = ts.tiles[0].vp, viewport = ts.opts.viewport;
  const shadow = { center: [5, 5, 0], radius: 8 };
  const log = {
    vpWithShadow: tryDraw([asBatch(b, { vp })], vp, { viewport, shadow }),
    badBlend: tryDraw([asBatch(b, { blend: 'screen' })], vp, { viewport }),
    shortVp: tryDraw([asBatch(b, { vp: vp.subarray(0, 12) })], vp, { viewport }),
    noVpAtAll: tryDraw([asBatch(b)], null, { viewport }),
    ppuWithShadow: tryDraw([asBatch(b, { pixelsPerUnit: 9 })], vp, { viewport, shadow }),
  };
  log.ok = !!log.vpWithShadow && !!log.badBlend && !!log.shortVp && !!log.noVpAtAll && log.ppuWithShadow === null;
  return log;
}
