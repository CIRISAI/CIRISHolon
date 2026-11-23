// A 2D stage drawn by holon-draw: the page-side helper the Canvas-2D pages were ported onto.
//
// NOT a copy of the engine. `./draw.js` and `./text.js` beside it are (README.md names the
// commit); this file is the pages' own, and adds nothing to the engine: it packs holon-draw's
// SCREEN records (the 52-byte record of draw.js's header) in CSS pixels, scales them to
// device pixels, groups consecutive records of one kind and one blend into one batch, and
// hands the frame to draw() in the order the page painted it. Every mark is a record of a
// kind the engine already has:
//   line  (2)   a straight segment, flat ends, width in px          -> canvas lineTo + stroke
//   disc  (4)   a filled circle                                     -> canvas arc + fill
//   quad  (1)   a rectangle with a corner radius                    -> canvas fillRect / roundRect
//   run   (11)  a text run from text.js's atlas                     -> canvas fillText
// and four things Canvas 2D has and the engine's screen kinds do not, built from those:
//   DASHES      a dashed path is cut into its dash segments here (setLineDash, continuous
//               along the path as Canvas does it, starting at the path's first point);
//   ROUND CAPS  a disc of the line's half width at each end (exact for an opaque line; a
//               translucent one blends twice where the cap overlaps the body);
//   CIRCLE STROKES  a polyline of short segments;
//   RADIAL GRADIENTS  a stack of discs, painted from the outer stop inward. A two-circle
//               radial gradient's contours are circles whose centre and radius move linearly
//               with t, so the stack is the gradient sampled at `steps` values of t: a
//               staircase of `steps` bands instead of a continuous ramp. With blend 'add'
//               (Canvas "lighter") each disc adds its band's share, so the sum is the same
//               staircase of the gradient's premultiplied colour.
// Hit-testing stays with each page (it never asked the canvas).

import { createDrawer, KIND, INSTANCE_BYTES } from "./draw.js";
import { createTextProvider } from "./text.js";

const IDENTITY = new Float32Array([1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]);

/** "#rrggbb" or "rgb(a)(r, g, b[, a])" -> [r, g, b, a] with r, g, b 0..255 and a 0..1. */
export function parseColour(c) {
  if (Array.isArray(c)) return c;
  const s = String(c).trim();
  if (s[0] === "#") {
    const h = s.length === 4 ? s.slice(1).split("").map((x) => x + x).join("") : s.slice(1);
    const n = parseInt(h.slice(0, 6), 16);
    return [(n >> 16) & 255, (n >> 8) & 255, n & 255, h.length === 8 ? parseInt(h.slice(6), 16) / 255 : 1];
  }
  const m = s.match(/rgba?\(([^)]+)\)/);
  if (m) {
    const p = m[1].split(",").map((x) => Number(x.trim()));
    return [p[0], p[1], p[2], p.length > 3 ? p[3] : 1];
  }
  throw new Error("screen.js: a colour it cannot read: " + c);
}

/**
 * @param {HTMLCanvasElement} canvas
 * @param {{ dpr: number, styles?: Array<object>, clear?: [number, number, number, number] }} options
 *   clear: the colour the frame starts from, 0..1 each; transparent black (the CSS background
 *   shows through, as it did through a cleared 2D canvas) when absent.
 */
export function createStage(canvas, options) {
  const gl = canvas.getContext("webgl2", { antialias: true }) || canvas.getContext("webgl", { antialias: true });
  if (!gl) throw new Error("this browser offers no WebGL, and the stage is drawn with it");
  let dpr = options.dpr;
  const text = createTextProvider(gl, { dpr, styles: options.styles || [] });
  const drawer = createDrawer(gl, { text });
  const clear = options.clear || [0, 0, 0, 0];

  let buf = new ArrayBuffer(INSTANCE_BYTES * 4096);
  let f32 = new Float32Array(buf), u8 = new Uint8Array(buf);
  let n = 0;                       // records this frame
  const spans = [];                // [kind, blend, first, count]
  let blend = "alpha";
  const batches = [], viewport = [0, 0, 0, 0], opts = { viewport };
  let calls = 0, records = 0;      // what the last frame handed draw.js

  function grow() {
    const nb = new ArrayBuffer(buf.byteLength * 2);
    new Uint8Array(nb).set(u8);
    buf = nb; f32 = new Float32Array(buf); u8 = new Uint8Array(buf);
  }

  // One record. Coordinates and lengths arrive in CSS px and are stored in device px.
  function rec(kind, x, y, s0, s1, a0, a1, a2, angle, rgba, variant) {
    if ((n + 1) * INSTANCE_BYTES > buf.byteLength) grow();
    const o = n * 13, b = n * INSTANCE_BYTES;
    f32[o] = x; f32[o + 1] = y; f32[o + 2] = 0;
    f32[o + 3] = s0; f32[o + 4] = s1; f32[o + 5] = 0;
    f32[o + 6] = a0; f32[o + 7] = a1; f32[o + 8] = a2; f32[o + 9] = angle;
    u8[b + 40] = rgba[0]; u8[b + 41] = rgba[1]; u8[b + 42] = rgba[2];
    u8[b + 43] = Math.max(0, Math.min(255, Math.round(rgba[3] * 255)));
    u8[b + 44] = 0; u8[b + 45] = 0; u8[b + 46] = 0; u8[b + 47] = 0;
    u8[b + 48] = kind; u8[b + 49] = 0; u8[b + 50] = 0; u8[b + 51] = variant | 0;
    const last = spans[spans.length - 1];
    // a run's atlas page is its variant: runs of different pages are different batches
    const key = kind === KIND.RUN ? 100 + (variant | 0) : kind;
    if (last && last[0] === key && last[1] === blend) last[3]++;
    else spans.push([key, blend, n, 1]);
    n++;
  }

  const api = {
    gl, text, drawer,
    get dpr() { return dpr; },
    /** Calls and records the last end() handed draw.js (draw.js's own stats() for the GPU side). */
    get lastFrame() { return { batches: calls, records, drawCalls: drawer.stats().drawCalls }; },
    setDpr(d) { if (d !== dpr) { dpr = d; text.setDpr(d); } },
    /** Start a frame: the canvas's backing store is the caller's (it sizes canvas.width/height). */
    begin() { n = 0; spans.length = 0; blend = "alpha"; text.beginFrame(); },
    /** 'alpha' (source-over) or 'add' (Canvas "lighter") for what follows. */
    blend(mode) { blend = mode; },

    line(x0, y0, x1, y1, width, colour) {
      const c = parseColour(colour);
      rec(KIND.LINE, x0 * dpr, y0 * dpr, width * dpr, 0, x1 * dpr, y1 * dpr, 0, 0, c, 0);
    },
    /** A line with round caps (Canvas lineCap "round"). */
    roundLine(x0, y0, x1, y1, width, colour) {
      const c = parseColour(colour);
      rec(KIND.LINE, x0 * dpr, y0 * dpr, width * dpr, 0, x1 * dpr, y1 * dpr, 0, 0, c, 0);
      rec(KIND.DISC, x0 * dpr, y0 * dpr, (width / 2) * dpr, 0, 0, 0, 0, 0, c, 0);
      rec(KIND.DISC, x1 * dpr, y1 * dpr, (width / 2) * dpr, 0, 0, 0, 0, 0, c, 0);
    },
    disc(x, y, r, colour) {
      rec(KIND.DISC, x * dpr, y * dpr, r * dpr, 0, 0, 0, 0, 0, parseColour(colour), 0);
    },
    /** A filled rectangle by its top-left corner, with a corner radius (0 square). */
    rect(x, y, w, h, colour, radius = 0) {
      rec(KIND.QUAD, (x + w / 2) * dpr, (y + h / 2) * dpr, w * dpr, h * dpr, radius * dpr, 0, 0, 0, parseColour(colour), 0);
    },
    /**
     * A polyline (points [[x, y], ...]) stroked with flat ends; `dash` [on, off, ...] cuts it
     * into dashes continuous along its length, as setLineDash does from the first point.
     * `joins` puts a disc of the half width at each interior point (for an opaque line, so a
     * bend has no notch; Canvas's default join is a miter, which this does not reproduce).
     * `roundCaps` (Canvas lineCap "round") puts that disc at each end, and at each dash's ends.
     */
    path(points, width, colour, dash = null, joins = false, closed = false, roundCaps = false) {
      const c = parseColour(colour);
      const pts = closed ? points.concat([points[0]]) : points;
      const cap = (x, y) => rec(KIND.DISC, x * dpr, y * dpr, (width / 2) * dpr, 0, 0, 0, 0, 0, c, 0);
      if (!dash || dash.length === 0) {
        for (let i = 0; i + 1 < pts.length; i++) {
          rec(KIND.LINE, pts[i][0] * dpr, pts[i][1] * dpr, width * dpr, 0, pts[i + 1][0] * dpr, pts[i + 1][1] * dpr, 0, 0, c, 0);
        }
        if (joins) for (let i = 1; i + 1 < pts.length; i++) cap(pts[i][0], pts[i][1]);
        if (roundCaps && !closed && pts.length > 1) { cap(pts[0][0], pts[0][1]); cap(pts[pts.length - 1][0], pts[pts.length - 1][1]); }
        return;
      }
      const pat = dash.length % 2 ? dash.concat(dash) : dash;
      let k = 0, left = pat[0];          // which entry of the pattern, and how much of it is left
      for (let i = 0; i + 1 < pts.length; i++) {
        const [ax, ay] = pts[i], [bx, by] = pts[i + 1];
        const len = Math.hypot(bx - ax, by - ay);
        let at = 0;
        while (at < len) {
          const step = Math.min(left, len - at);
          if (k % 2 === 0 && step > 0) {
            const t0 = at / len, t1 = (at + step) / len;
            rec(KIND.LINE, (ax + (bx - ax) * t0) * dpr, (ay + (by - ay) * t0) * dpr, width * dpr, 0,
              (ax + (bx - ax) * t1) * dpr, (ay + (by - ay) * t1) * dpr, 0, 0, c, 0);
            // (a dash cut at a corner gets caps at the corner too; Canvas joins it there)
            if (roundCaps) { cap(ax + (bx - ax) * t0, ay + (by - ay) * t0); cap(ax + (bx - ax) * t1, ay + (by - ay) * t1); }
          }
          at += step; left -= step;
          if (left <= 1e-9) { k = (k + 1) % pat.length; left = pat[k]; }
        }
      }
    },
    /** Canvas strokeRect: the four sides as one closed path (dashes run round it from the top-left). */
    strokeRect(x, y, w, h, width, colour, dash = null, roundCaps = false) {
      api.path([[x, y], [x + w, y], [x + w, y + h], [x, y + h]], width, colour, dash, false, true, roundCaps);
    },
    /** Canvas roundRect(x, y, w, h, r) + stroke: the outline as a closed polyline, 6 pieces a corner. */
    strokeRoundRect(x, y, w, h, r, width, colour) {
      const pts = [];
      const corner = (cx, cy, a0) => { for (let i = 0; i <= 6; i++) { const a = a0 + (i / 6) * (Math.PI / 2); pts.push([cx + r * Math.cos(a), cy + r * Math.sin(a)]); } };
      corner(x + w - r, y + r, -Math.PI / 2); corner(x + w - r, y + h - r, 0);
      corner(x + r, y + h - r, Math.PI / 2); corner(x + r, y + r, Math.PI);
      api.path(pts, width, colour, null, false, true);
    },
    /** Canvas arc(x, y, r, 0, 2pi) + stroke: a closed polyline of `segments` pieces. */
    strokeCircle(x, y, r, width, colour, dash = null, segments = 0) {
      const m = segments || Math.max(48, Math.min(256, Math.ceil(r * dpr * 0.75)));
      const pts = [];
      for (let i = 0; i < m; i++) { const a = (i / m) * Math.PI * 2; pts.push([x + r * Math.cos(a), y + r * Math.sin(a)]); }
      api.path(pts, width, colour, dash, false, true);
    },
    /**
     * Canvas createRadialGradient(x0, y0, r0, x1, y1, r1) with colour stops [[t, colour], ...],
     * filling arc(cx, cy, R): the disc stack described at the top. Painted with the current
     * blend: 'alpha' paints opaque-to-translucent bands over one another from t = 1 inward
     * (exact for stops that are opaque); 'add' adds each band's share of the premultiplied
     * colour, so the sum is the gradient's (as "lighter" composes it).
     * The fill is clipped to arc(cx, cy, R) by Canvas; here the stack's outer disc IS that arc
     * when (x1, y1, r1) is (cx, cy, R), which is how every caller uses it.
     */
    radial(x0, y0, r0, x1, y1, r1, stops, steps = 16) {
      const cs = stops.map(([t, col]) => [t, parseColour(col)]);
      const at = (t) => {
        if (t <= cs[0][0]) return cs[0][1];
        for (let i = 1; i < cs.length; i++) {
          if (t <= cs[i][0]) {
            const [ta, ca] = cs[i - 1], [tb, cb] = cs[i], u = (t - ta) / Math.max(1e-9, tb - ta);
            return [0, 1, 2, 3].map((j) => ca[j] + (cb[j] - ca[j]) * u);
          }
        }
        return cs[cs.length - 1][1];
      };
      const circle = (t) => [x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, r0 + (r1 - r0) * t];
      if (blend === "add") {
        // band k covers t in [k/steps, (k+1)/steps); the disc of contour (k+1)/steps adds
        // s_k - s_(k+1) of premultiplied colour P, so a point in band j sums to s_j
        const pm = (t) => { const c = at(t); return [c[0] * c[3], c[1] * c[3], c[2] * c[3]]; };
        for (let k = steps - 1; k >= 0; k--) {
          // the band's sample is its middle, s_k = P((k + 0.5) / steps), and s_steps = P(1)
          const [cx, cy, r] = circle((k + 1) / steps);
          const a = pm((k + 0.5) / steps), b = k + 1 < steps ? pm((k + 1.5) / steps) : pm(1);
          const d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]].map((v) => Math.max(0, v));
          const m = Math.max(d[0], d[1], d[2]);
          if (m <= 0 || r <= 0) continue;
          // colour * alpha = d, with the colour as bright as a byte holds
          const alpha = Math.min(1, m / 255);
          rec(KIND.DISC, cx * dpr, cy * dpr, r * dpr, 0, 0, 0, 0, 0, [d[0] / alpha, d[1] / alpha, d[2] / alpha, alpha], 0);
        }
        // past contour 1 (inside the arc) Canvas paints the last stop: its own disc, so a
        // point in band j sums to s_j - P(1) + P(1); inside the inner circle that is s_0,
        // about the first stop, which Canvas paints there
        const last = pm(1), lm = Math.max(last[0], last[1], last[2]);
        if (lm > 0 && r1 > 0) {
          const alpha = Math.min(1, lm / 255);
          rec(KIND.DISC, x1 * dpr, y1 * dpr, r1 * dpr, 0, 0, 0, 0, 0, [last[0] / alpha, last[1] / alpha, last[2] / alpha, alpha], 0);
        }
        return;
      }
      for (let k = steps; k >= 0; k--) {
        const t = k / steps, [cx, cy, r] = circle(t);
        if (r <= 0) continue;
        // disc k is contour t = k / steps; what shows of it is the band just inside it
        const c = at(k === 0 ? 0 : (k - 0.5) / steps);
        rec(KIND.DISC, cx * dpr, cy * dpr, r * dpr, 0, 0, 0, 0, 0, c, 0);
      }
    },
    /**
     * Canvas fillText. align 'left' | 'center' | 'right'; baseline 'middle' centres the run's
     * line box on y, 'alphabetic' puts y about where Canvas's baseline is (the box's centre
     * raised by a third of the line box, the provider's ascent share at these sizes).
     */
    text(str, style, x, y, colour, align = "left", baseline = "middle") {
      const r = text.place(String(str), style);
      if (!r || r.w === 0) return;
      const c = parseColour(colour);
      let cx = x * dpr;
      if (align === "left") cx += r.w / 2 - 1;
      else if (align === "right") cx -= r.w / 2 - 1;
      let cy = y * dpr;
      if (baseline === "alphabetic") cy -= r.h * 0.3;
      else if (baseline === "top") cy += r.h / 2;
      rec(KIND.RUN, cx, cy, r.w, r.h, r.u0, r.v0, r.u1, r.v1, c, r.page);
      return r.advance / dpr;
    },
    /** Draw the frame: clear to `clear`, then every record in the order it was painted. */
    end(width, height) {
      const W = width | 0, H = height | 0;
      gl.viewport(0, 0, W, H);
      viewport[2] = W; viewport[3] = H;
      gl.clearColor(clear[0], clear[1], clear[2], clear[3]);
      gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
      batches.length = 0;
      for (const [key, bl, first, count] of spans) {
        const kind = key >= 100 ? KIND.RUN : key;
        const bt = { kind, offset: [0, 0, 0], count, screen: true, bytes: new Uint8Array(buf, first * INSTANCE_BYTES, count * INSTANCE_BYTES) };
        if (bl === "add") bt.blend = "add";
        batches.push(bt);
      }
      calls = batches.length; records = n;
      if (batches.length) drawer.draw(batches, IDENTITY, opts);
    },
  };
  return api;
}
