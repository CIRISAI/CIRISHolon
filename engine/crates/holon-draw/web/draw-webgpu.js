// holon-draw, the WebGPU backend: the sibling of `./draw.js`, with THE SAME CONTRACT.
//
// One ES module, no dependency. The record, the kinds, screen batches, tile settings, colour
// modes, surface classes, line ends and shape, the field, oriented boxes, bent arcs,
// materials, shadows, `shade`, `coarse`, `offset`, `flatLines`, the flat-map mode and
// `pixelsPerUnit` are specified once, in the header of `./draw.js`, and are not repeated
// here. A consumer switches backends without changing a batch. What differs is how the
// backend is hosted, and that is all this header specifies.
//
//   const drawer = await createDrawerWebGPU(device, { format, text, sampleCount: 4 });
//   each frame:  drawer.draw(batches, vp, { ...the options of draw.js, target, clear });
//
// CREATION. `createDrawerWebGPU(device, options)` is async (its pipelines are compiled with
// createRenderPipelineAsync, and every shader module's compilation messages are read; an
// error rejects the promise with the messages). Options:
//   format        the colour target's format (required), e.g. navigator.gpu.getPreferredCanvasFormat()
//   text          a provider from `createTextProviderWebGPU` in `./text.js` (texture(page) is a
//                 GPUTexture); without one, run batches are skipped
//   sampleCount   1 (default) or 4. With 4 the drawer owns a 4x colour texture, draws into it
//                 and resolves it into the target (this is what a WebGL canvas made with
//                 `antialias: true` does)
//   depthFormat   the depth format of the pass, 'depth24plus' (default); it must match
//                 `opts.depthTarget` when the host gives one
//   sphereSegments, rodSides   as in draw.js
//
// draw(batches, vp, opts) takes every option draw.js takes, and:
//   target        a GPUTexture or a GPUTextureView to draw into (required). It is the resolve
//                 target when sampleCount is 4.
//   size          [w, h] of the target in pixels. Needed only when `target` is a view (a view
//                 does not say its size) and no `viewport` is given; otherwise it is the
//                 texture's size, or the viewport's far corner.
//   depthTarget   a GPUTextureView of depthFormat (and of sampleCount); without it the drawer
//                 keeps its own, sized to the target and made again when that size changes.
//   clear         [r, g, b, a]: clear the colour (and the depth to 1) before drawing.
//                 Without it the pass LOADS what is there: the target's pixels at sampleCount
//                 1; at sampleCount 4 the drawer's own 4x picture from the previous draw()
//                 (which the host cannot see or write), so a host at 4x passes `clear` on the
//                 first draw() of each frame.
//   viewport      [x, y, w, h] AS GL HAS IT (y from the bottom), the same numbers draw.js
//                 takes; default the whole target. The pass's viewport is set from it.
// RESIDENT BUFFERS: resident(bytes) -> handle, update(handle, bytes), release(handle) and the
// batch { kind, handle, first, count, ..., slot? } are draw.js's (its header specifies them,
// the resident image batch's `slot` and every throw included). Here a handle is one GPUBuffer
// (usage VERTEX | COPY_DST), written with queue.writeBuffer, and a resident batch is drawn by
// setVertexBuffer(1, that buffer, first * 52): no copy, no upload, bytesUploaded 0. update()
// writes in place when the records fit the buffer, else makes a new GPUBuffer for the same
// handle and destroys the old one (work already submitted that reads it still completes).
// Since queue.writeBuffer is ordered with submits, an update() between two draw() calls is
// what the second one reads. release() destroys the GPUBuffer. There are no vertex arrays to
// cache: a resident batch costs the same commands as a `bytes` batch, less its writeBuffer.
//
// draw() records one command buffer (the shadow pass first, when asked for, then one render
// pass with the world and the screen batches) and submits it. It returns nothing. A draw()
// with nothing to draw submits nothing, unless `clear` is given (then it clears).
//
// WHAT IS THE SAME AS draw.js, ON PURPOSE: the meshes (the builders below are COPIES of
// draw.js's, byte for byte; tools/webgpu_parity.py checks that they have not drifted), the
// slots and their modes, the order of passes and of batches, the frame state (blending
// SRC_ALPHA, ONE_MINUS_SRC_ALPHA for colour and ONE, ONE_MINUS_SRC_ALPHA for alpha; depth
// LEQUAL with writes; back faces culled for closed meshes; the depth test off for screen
// batches and under `depthTest: false`), the eye from vp, the light's matrix, and every
// formula of the vertex and fragment stages (WGSL ports of draw.js's GLSL).
//
// WHAT DIFFERS, AND WHY:
//   - Clip space. vp is the GL matrix the host already has (z in [-w, w]); the vertex stage
//     writes z' = (z + w) / 2, so the depth stored is GL's window depth, bit for bit in its
//     formula, and the shadow test compares the same numbers.
//   - Precision. WGSL has f32 in every stage, so this backend is always draw.js's HIGH path
//     (the twin's hash). There is no mediump path and no `forceMediump`.
//   - The shadow map is a depth32float depth texture read with textureLoad (four taps,
//     nearest, clamped to the edge, as the GL map is sampled), not depth packed in RGBA8.
//     Its row order is WebGPU's (row 0 at the top), so the lookup flips v.
//   - Uniforms. Each draw call reads its own 256-byte slot of one uniform buffer (dynamic
//     offsets), written once per draw() with one queue.writeBuffer. `paramUploads` in
//     stats() counts what draw.js would have uploaded (tile settings that differ from the
//     previous draw of the same pass), so the two backends' counts can be compared;
//     `uniformBytes` is what this backend actually wrote.
//   - Pipelines. The world pipeline exists in variants by (shadows on or off, culling,
//     depth test on or off), each made on first use; without `opts.shadow` no shadow
//     variant is ever made and no draw pays for shadow code (the same split as HD_SHADOW).
//   - Text. The atlas pages are r8unorm textures (coverage in red) sampled LINEAR, clamped,
//     with textureSampleLevel(level 0): the same filtering as the GL ALPHA pages.
//   - Glyph shapes and the FLOW dash are not ported by hand: draw.js's GLYPH_GLSL (written in
//     a small subset of GLSL for this) is translated to WGSL line for line by wgslOf below.
//   - The RING's per-pixel distance. A fragment's position here is the target's pixel from
//     the TOP-left (GL's gl_FragCoord is from the bottom-left), so the frame carries the
//     viewport's top-left in the target (Frame.org: opts.viewport's x and h - y - height) and
//     the vertex stage hands over the ring's centre in those pixels. In the SHADOW pass's frame,
//     Frame.lvp holds the CAMERA's vp and Frame.view.xy the camera's viewport size (the pass's
//     own vp is in Frame.vp), so the ring is the camera's there too, as in draw.js.
//   - Images (kind 12). setImage(slot, source, { nearest }) and deleteImage(slot), as in
//     draw.js: each slot is an rgba8unorm texture written with copyExternalImageToTexture
//     (not premultiplied, not flipped), sampled nearest or LINEAR, clamped, level 0. A slot is
//     bind group 2; a draw with no image binds a 1x1 blank there.
//
// STATE. This drawer touches no host object but the target (and depthTarget). It owns its
// buffers, textures and pipelines; destroy() frees them.
//
// Not handled, as in draw.js: a `vp` that mirrors the picture flips winding.

import { KIND, INSTANCE_BYTES, PARAMS_NEUTRAL, GLYPH_GLSL } from './draw.js';
export {
  KIND, INSTANCE_BYTES, PARAM, PARAMS_NEUTRAL, COLOUR, VARIANT_CASING, VARIANT_ROUND, VARIANT_ORIENT,
  VARIANT_MATERIAL, MATERIAL, DISC_CASING, DISC_CANOPY_FIELD, VARIANT_FLOW, VARIANT_GLYPH_SHAPE, GLYPH,
} from './draw.js';

// draw.js's glyph shapes and flow dash (GLYPH_GLSL, a small subset of GLSL written for this)
// in WGSL, line for line: a function header `T hd_x(T a, ...) {` becomes `fn hd_x(a: T, ...) -> T {`,
// a local `T x = ` becomes `var x = `, and vecN( becomes vecNf(. Nothing else differs, so the
// two backends draw the shapes from one source.
function wgslOf(src) {
  const T = { float: 'f32', vec2: 'vec2f', vec3: 'vec3f', vec4: 'vec4f' };
  return src.split('\n').map((line) => {
    const m = line.match(/^(float|vec2|vec3|vec4) (hd_\w+)\((.*)\) \{$/);
    if (m) {
      return 'fn ' + m[2] + '(' + m[3].split(', ').map((a) => { const [t, n] = a.split(' '); return n + ': ' + T[t]; }).join(', ') + ') -> ' + T[m[1]] + ' {';
    }
    return line.replace(/^(\s*)(float|vec2|vec3|vec4) (\w+) = /, '$1var $3 = ').replace(/\bvec([234])\(/g, 'vec$1f(');
  }).join('\n');
}
const GLYPH_WGSL = wgslOf(GLYPH_GLSL);

const MAX_PAGES = 4;
const FRAME_FLOATS = 52;   // struct Frame: 208 bytes
const DRAW_FLOATS = 44;    // struct Draw: 176 bytes

// ---- WGSL ----

const COMMON = /* wgsl */ `
struct Frame {
  vp: mat4x4f,     // the view-projection (GL clip convention)
  lvp: mat4x4f,    // the light's view-projection (shadows)
  eye: vec4f,      // homogeneous eye: xyz - p*w points from p to the viewer
  light: vec4f,    // xyz: unit, toward the light
  view: vec4f,     // x, y: the viewport's width and height in pixels; z: half its height; w: pixels per world unit if given, else 0
  shp: vec4f,      // x strength, y one texel, z bias, w the normal offset in world units
  org: vec4f,      // x, y: the viewport's top-left in the target's pixels (WebGPU's, from the top), for the RING
};
struct Draw {
  params: array<vec4f, 8>,   // the tile's settings
  offset: vec4f,
  mode: vec4f,     // world: x vertex mode (0 sphere, 1 tube, 2 scaled-and-turned, 3 bent tube), y radius scale, z kind code
                   // screen: x mode (0 rectangle, 1 circle, 2 line, 3 run or image), w 1 for a run, 2 for an image
  shade: vec4f,    // x lit (0 or 1), y ambient, z specular, w rim
};
@group(0) @binding(0) var<uniform> F: Frame;
@group(0) @binding(1) var<uniform> D: Draw;
struct VIn {
  @location(0) a_p: vec3f,
  @location(1) a_n: vec3f,
  @location(2) i_pos: vec3f,
  @location(3) i_size: vec3f,
  @location(4) i_aux: vec3f,
  @location(5) i_angle: f32,
  @location(6) i_color: vec4f,
  @location(7) i_meta: vec4u,  // kind, detail, flags, variant
};
`;

// zpass: the shadow map's vertex stage, whose ring is the CAMERA's (F.lvp holds the camera's
// view-projection and F.view.xy its viewport's size in that frame; see draw()).
function worldVS(shadow, zpass) {
  return /* wgsl */ `
struct VOut {
  @builtin(position) pos: vec4f,
  @location(0) n: vec3f,
  @location(1) v: vec3f,
  @location(2) c: vec4f,
  @location(3) dim: vec4f,     // rgb: DIM; a: the alpha multiplier (class times FADE)
  @location(4) np: vec2f,      // the field's position
  @location(5) fp: vec3f,      // pixels per field unit; the field's amplitude (0 off); which field (0 ground, 1 canopy, 2 roof)
  @location(6) ring: vec3f,    // draw.js's v_ring (picture: the centre in the target's pixels, here from the top)
  @location(7) mat: f32,       // the material (0 kind, 1 matte, 2 metal, 3 glow)
${shadow ? '  @location(8) ls: vec4f,      // the position in the light\'s clip space\n' : ''}  @location(9) sp: vec2f,      // draw.js's v_sp: a glyph's shape and half size in pixels; 16 and a flow's upper clamp
};
// Pixels per world unit at clip w.
fn ppuAt(w: f32) -> f32 {
  if (F.view.w > 0.0) { return F.view.w; }
  return F.view.z * length(vec3f(F.vp[0][1], F.vp[1][1], F.vp[2][1])) / max(w, 1e-6);
}
@vertex fn vs(a: VIn) -> VOut {
  var o: VOut;
  let kc = D.mode.z;
  let mx = D.mode.x;
  let vr = a.i_meta.w;
  let bit0 = (vr & 1u) != 0u;
  let bit1 = (vr & 2u) != 0u;
  let bit2 = (vr & 4u) != 0u;
  let orient = kc > 4.5 && kc < 5.5 && (vr & 8u) != 0u;
  let liney = kc > 2.5 && kc < 3.5;
  let shape = f32(vr & 15u);
  let bill = kc > 5.5 && shape > 0.5;   // a glyph of shape 1..15: a camera-facing square
  let flow = liney && bit2;             // a FLOW line or arc
  var gq = vec2f(0.0);
  var fU = 0.0;
  var fLen = 1.0;
  var fC = vec3f(0.0);
  var p: vec3f;
  var n: vec3f;
  let org = a.i_pos + D.offset.xyz;
  if (mx < 0.5) {
    n = a.a_n;
    p = org + a.a_p * a.i_size.x;
  } else if (mx < 1.5 || mx > 2.5) {
    let d = a.i_aux - a.i_pos;
    // Along the axis: t in [0, 1] on the body; a cap's vertices lie beyond, by e radii.
    let t = clamp(a.a_p.z, 0.0, 1.0);
    var c0 = org + d * t;
    var dd = d;
    if (mx > 2.5) {
      // An arc: the quadratic Bezier o -> ctrl -> o + d, ctrl from size[1] (up) and size[2] (across).
      let hz = vec3f(d.xy, 0.0);
      let hl = length(hz);
      var across = vec3f(0.0, 1.0, 0.0);
      if (hl > 1e-20) { across = vec3f(-hz.y, hz.x, 0.0) / hl; }
      let ctrl = 0.5 * d + vec3f(0.0, 0.0, a.i_size.y) + across * a.i_size.z;
      let u = 1.0 - t;
      c0 = org + 2.0 * u * t * ctrl + t * t * d;
      dd = 2.0 * u * ctrl + 2.0 * t * (d - ctrl);
      if (flow) {
        // the length along the curve to t and to 1 (Simpson's rule, 8 intervals each)
        var sT = 0.0;
        var s1 = 0.0;
        for (var k = 0; k <= 8; k++) {
          var wk = 2.0;
          if (k == 0 || k == 8) { wk = 1.0; } else if ((k & 1) == 1) { wk = 4.0; }
          let a1 = f32(k) / 8.0;
          let aT = a1 * t;
          s1 += wk * length(2.0 * (1.0 - a1) * ctrl + 2.0 * a1 * (d - ctrl));
          sT += wk * length(2.0 * (1.0 - aT) * ctrl + 2.0 * aT * (d - ctrl));
        }
        fU = t;
        if (s1 > 0.0) { fU = sT * t / s1; }
        fLen = s1 / 24.0;
      }
    } else if (flow) {
      fU = t;
      fLen = length(d);
    }
    let len = length(dd);
    var dn = vec3f(0.0, 0.0, 1.0);
    if (len > 1e-20) { dn = dd / len; }
    var up = vec3f(0.0, 0.0, 1.0);
    if (!(abs(dn.z) < 0.999)) { up = vec3f(1.0, 0.0, 0.0); }
    let bx = normalize(cross(up, dn));
    let by = cross(dn, bx);
    var r = a.i_size.x * D.mode.y;
    var e = 0.0;
    if (liney && bit1) { e = a.a_p.z - t; }
    var fl = 0.0;
    if (liney) { fl = select(D.params[6].x, D.params[5].y, bit0); }
    if (fl > 0.0) {
      // the street floor: at least this many pixels of half width, measured on the centre line
      r = max(r, fl / ppuAt((F.vp * vec4f(c0, 1.0)).w));
    }
    p = c0 + (bx * a.a_p.x + by * a.a_p.y) * r + dn * (e * r);
    n = bx * a.a_n.x + by * a.a_n.y + dn * a.a_n.z;
    if (flow) {
      fU += e * r / max(fLen, 1e-6);
      fC = c0;
    }
  } else if (bill) {
    // the camera's right and up from vp alone (draw.js says how)
    let r0 = vec3f(F.vp[0][0], F.vp[1][0], F.vp[2][0]);
    let r1 = vec3f(F.vp[0][1], F.vp[1][1], F.vp[2][1]);
    let r3 = vec3f(F.vp[0][3], F.vp[1][3], F.vp[2][3]);
    var rz = vec3f(F.vp[0][2], F.vp[1][2], F.vp[2][2]);
    if (dot(r3, r3) > 1e-12) { rz = r3; }
    var R = normalize(cross(r1, rz));
    if (dot(R, r0) < 0.0) { R = -R; }
    var U = normalize(cross(rz, r0));
    if (dot(U, r1) < 0.0) { U = -U; }
    gq = a.a_p.xy / max(max(abs(a.a_p.x), abs(a.a_p.y)), 1e-6);
    let c = cos(a.i_angle);
    let sn = sin(a.i_angle);
    let cr = vec2f(c * gq.x - sn * gq.y, sn * gq.x + c * gq.y) * a.i_size.x;
    p = org + R * cr.x + U * cr.y;
    n = normalize(cross(R, U));
  } else {
    var s = a.i_size;
    if (D.mode.y > 0.5) { s = a.i_size.xxx; }
    let c = cos(a.i_angle);
    let sn = sin(a.i_angle);
    let q = a.a_p * s;
    if (orient) {
      let al = length(a.i_aux);
      var ez = vec3f(0.0, 0.0, 1.0);
      if (al > 1e-20) { ez = a.i_aux / al; }
      var r0 = vec3f(1.0, 0.0, 0.0);
      if (!(abs(ez.x) < 0.999)) { r0 = vec3f(0.0, 1.0, 0.0); }
      let x0 = normalize(r0 - ez * dot(r0, ez));
      let y0 = cross(ez, x0);
      let ex = c * x0 + sn * y0;
      let ey = cross(ez, ex);
      p = org + ex * q.x + ey * q.y + ez * q.z;
      n = ex * a.a_n.x + ey * a.a_n.y + ez * a.a_n.z;
    } else {
      p = org + vec3f(c * q.x - sn * q.y, sn * q.x + c * q.y, q.z);
      n = vec3f(c * a.a_n.x - sn * a.a_n.y, sn * a.a_n.x + c * a.a_n.y, a.a_n.z);
    }
  }
  o.n = n;
  o.v = normalize(F.eye.xyz - p * F.eye.w);
  let cm = vr >> 6u;
  let ramp = cm == 1u;
  let tint = cm == 2u;
  let tube = (mx > 0.5 && mx < 1.5) || mx > 2.5;
  var rgb = a.i_color.rgb;
  var dens = 0.0;
  if (ramp) {
    dens = clamp(select(select(a.i_aux.x, 0.0, orient), a.i_angle, tube), 0.0, 1.0);
    rgb = mix(D.params[1].rgb, D.params[2].rgb, dens);
  } else if (tint) {
    rgb = rgb * D.params[3].rgb;
  }
  // The surface class: its alpha multiplier (1 for a mark in no class) and its field (-1 none).
  var am = 1.0;
  var fk = -1.0;
  var fa = 1.0;
  if ((kc > 1.5 && kc < 2.5) || (kc > 0.5 && kc < 1.5 && ramp)) {
    am = D.params[6].y; fk = 0.0; fa = 1.0 - 0.5 * dens;               // ground
  } else if (tint && ((kc > 0.5 && kc < 1.5) || (kc > 4.5 && kc < 5.5))) {
    am = D.params[7].x; fk = 2.0;                                      // roof
  } else if (kc > 0.5 && kc < 1.5) {
    am = D.params[6].z;                                                // a literal quad: a shadow
  } else if (kc > 2.5 && kc < 3.5) {
    am = select(D.params[6].w, D.params[6].z, bit0);                   // casing, road
  } else if (kc > 3.5 && kc < 4.5) {
    if (bit0) { am = D.params[6].z; }                                  // a disc in the casing class
    if (bit2) { fk = 1.0; }                                            // a disc with the canopy field
  }
  let pos = F.vp * vec4f(p, 1.0);
${shadow ? '  o.ls = F.lvp * vec4f(p + n * F.shp.w, 1.0);\n' : ''}  o.mat = f32((vr >> 4u) & 3u);
  let ringR = D.params[7].w;
${zpass ? `  if (ringR > 0.0) {
    let cc = F.lvp * vec4f(p, 1.0);
    o.ring = vec3f((0.5 * F.view.x * (cc.x + cc.w) - D.params[7].y * cc.w) / ringR,
                   (0.5 * F.view.y * (cc.w - cc.y) - D.params[7].z * cc.w) / ringR, cc.w);
  } else {
    o.ring = vec3f(0.0, 0.0, 1.0);
  }` : `  if (ringR > 0.0) {
    o.ring = vec3f(F.org.x + D.params[7].y, F.org.y + D.params[7].z, ringR);
  } else {
    o.ring = vec3f(0.0);
  }`}
  o.c = vec4f(rgb, a.i_color.a);
  o.dim = vec4f(D.params[0].rgb, am * D.params[0].w);
  var amp = D.params[5].x * fa;
  if (fk < -0.5) { amp = 0.0; }
  if (amp > 0.0) {
    let pl = (p.xy - org.xy) + a.i_pos.xy;
    o.np = D.params[4].xy + D.params[4].zw * pl;
    o.fp = vec3f(ppuAt(pos.w) / max(abs(D.params[4].z), 1e-6), amp, fk);
  } else {
    o.np = vec2f(0.0);
    o.fp = vec3f(0.0);
  }
  o.sp = vec2f(0.0);
  if (bill) {
    o.np = gq;
    o.sp = vec2f(shape, a.i_size.x * ppuAt((F.vp * vec4f(org, 1.0)).w));
  } else if (flow) {
    var fN = 6.0;
    if (D.params[5].w > 0.0) { fN = D.params[5].w; }
    let ph = fract(D.params[5].z);
    o.np = vec2f(fU * fN - ph, -ph);
    o.sp = vec2f(16.0, fN - ph);
    o.fp = vec3f(fLen * ppuAt((F.vp * vec4f(fC, 1.0)).w) / fN, 0.0, 0.0);
  }
  // GL's clip z in [-w, w] to WebGPU's [0, w]: the same window depth.
  o.pos = vec4f(pos.x, pos.y, 0.5 * (pos.z + pos.w), pos.w);
  return o;
}
`;
}

function worldFS(shadow) {
  return /* wgsl */ `
${shadow ? '@group(1) @binding(0) var shadowMap: texture_depth_2d;\n' : ''}${GLYPH_WGSL}
// GLSL's mod: x - m * floor(x / m).
fn gmod(x: vec2f, m: f32) -> vec2f { return x - m * floor(x / m); }
// Value noise on a lattice that repeats every 4096 units; the twin's hash on mod(i, m) / m.
fn h2(i: vec2f, m: f32) -> f32 {
  let q = gmod(i, m) / m;
  return fract(sin(dot(q, vec2f(12.9898, 78.233)) * m) * 43758.5453);
}
fn vn(x: vec2f, cell: f32) -> f32 {
  let g = x / cell;
  let i = floor(g);
  var t = fract(g);
  t = t * t * (3.0 - 2.0 * t);
  let m = 4096.0 / cell;
  return mix(mix(h2(i, m), h2(i + vec2f(1.0, 0.0), m), t.x),
             mix(h2(i + vec2f(0.0, 1.0), m), h2(i + vec2f(1.0, 1.0), m), t.x), t.y) * 2.0 - 1.0;
}
// An octave, faded out between 4 and 2 pixels a cycle.
fn oct(x: vec2f, cell: f32, amp: f32, ppf: f32) -> f32 {
  let w = 1.0 - smoothstep(0.25, 0.5, 1.0 / (ppf * cell));
  if (w > 0.0) { return amp * w * vn(x, cell); }
  return 0.0;
}
${shadow ? `
fn tap(uv: vec2f, z: f32) -> f32 {
  let size = vec2i(textureDimensions(shadowMap));
  // nearest, clamped to the edge; v flipped (row 0 is the top in WebGPU, the bottom in GL)
  let tc = clamp(vec2i(floor(vec2f(uv.x, 1.0 - uv.y) * vec2f(size))), vec2i(0), size - vec2i(1));
  return select(1.0, 0.0, z > textureLoad(shadowMap, tc, 0));
}
// The lit fraction at this pixel: 1 in light, 0 in full shadow; four taps on a rotated grid.
fn litFraction(ls: vec4f, ndl: f32) -> f32 {
  let q = ls.xyz / ls.w * 0.5 + 0.5;
  if (q.x < 0.0 || q.x > 1.0 || q.y < 0.0 || q.y > 1.0 || q.z > 1.0) { return 1.0; }
  let bias = F.shp.z * (1.0 + 3.0 * (1.0 - ndl));
  let z = q.z - bias;
  let t = F.shp.y;
  let s = tap(q.xy + vec2f(-0.4, -1.2) * t, z) + tap(q.xy + vec2f(1.2, -0.4) * t, z)
        + tap(q.xy + vec2f(0.4, 1.2) * t, z) + tap(q.xy + vec2f(-1.2, 0.4) * t, z);
  return s * 0.25;
}
` : ''}
@fragment fn fs(i: VOut, @builtin(front_facing) ff: bool) -> @location(0) vec4f {
  // the ring, from this fragment's own position in the target
  var ring = 1.0;
  if (i.ring.z > 0.0) {
    ring = 1.0 - smoothstep(0.85, 1.0, length(min(abs(i.pos.xy - i.ring.xy) / i.ring.z, vec2f(8.0))));
  }
  let gk = floor(i.sp.x + 0.5);
  if (gk > 0.5 && gk < 15.5) {
    let g = hd_glyphCov(gk, i.np, i.sp.y);
    let cov = max(g.x, g.y);
    if (cov < 0.5) { discard; }
    return vec4f(mix(i.c.rgb, hd_ink(i.c.rgb), g.y) * i.dim.rgb, i.c.a * i.dim.a * ring * cov);
  }
  var n = normalize(i.n);
  if (!ff) { n = -n; }
  let v = normalize(i.v);
  let L = F.light.xyz;
  let ndl = max(dot(n, L), 0.0);
  let glow = D.shade.x > 0.5 && i.mat > 2.5;
${shadow ? `  var shf = 1.0;
  if (!glow) { shf = litFraction(i.ls, select(1.0, ndl, D.shade.x > 0.5)); }
  let strength = F.shp.x;
` : `  let shf = 1.0;
  let strength = 0.0;
`}  let sky = 0.75 + 0.25 * n.z;
  var lit = D.shade.y * sky + (1.0 - D.shade.y) * ndl * shf;
  let h = normalize(L + v);
  var spec = D.shade.z;
  var shin = 40.0;
  var rimk = D.shade.w;
  if (i.mat > 0.5 && i.mat < 1.5) { spec = 0.0; rimk = rimk * 0.25; }
  else if (i.mat > 1.5 && i.mat < 2.5) { spec = min(1.0, spec * 3.0 + 0.15); shin = 90.0; rimk = rimk * 1.6 + 0.06; lit = lit * 0.85; }
  let sp = pow(max(dot(n, h), 0.0), shin) * spec * step(0.0001, ndl) * shf;
  let rim = pow(1.0 - max(dot(n, v), 0.0), 3.0) * rimk;
  var base = i.c.rgb;
  var flowA = 1.0;
  if (gk > 15.5) {
    let fl = hd_flow(clamp(i.np.x, i.np.y, i.sp.y), clamp(1.5 / max(i.fp.x, 0.001), 0.02, 0.25));
    base = mix(base, vec3f(1.0), fl.x);
    flowA = fl.y;
  }
  if (i.fp.y > 0.0) {
    var s: f32;
    if (i.fp.z < 0.5) { s = oct(i.np, 48.0, 0.10, i.fp.x) + oct(i.np, 12.0, 0.07, i.fp.x) + oct(i.np, 3.0, 0.05, i.fp.x); }
    else if (i.fp.z < 1.5) { s = oct(i.np, 6.0, 0.16, i.fp.x) + oct(i.np, 1.5, 0.10, i.fp.x); }
    else { s = oct(i.np, 4.0, 0.05, i.fp.x) + oct(i.np, 1.0, 0.04, i.fp.x); }
    base = base * (1.0 + s * i.fp.y);
  }
  let shaded = select(base * (lit + rim) + vec3f(sp), base, glow);
  let flatc = base * mix(1.0 - strength, 1.0, shf);
  return vec4f(mix(flatc, shaded, D.shade.x) * i.dim.rgb, i.c.a * i.dim.a * ring * flowA);
}
`;
}

// The shadow map's fragment stage: depth only. A mark under half alpha, or a GLOW mark, casts
// nothing; the ring is the camera's at this fragment's point (behind the eye: outside).
const ZFS = /* wgsl */ `
@fragment fn zfs(i: VOut) {
  var ring = 0.0;
  if (i.ring.z > 0.0) { ring = 1.0 - smoothstep(0.85, 1.0, length(min(abs(i.ring.xy / i.ring.z), vec2f(8.0)))); }
  if (i.c.a * i.dim.a * ring < 0.5 || i.mat > 2.5) { discard; }
}
`;

const SCREEN = /* wgsl */ `
${GLYPH_WGSL}
@group(1) @binding(0) var t0: texture_2d<f32>;
@group(1) @binding(1) var t1: texture_2d<f32>;
@group(1) @binding(2) var t2: texture_2d<f32>;
@group(1) @binding(3) var t3: texture_2d<f32>;
@group(1) @binding(4) var smp: sampler;
@group(2) @binding(0) var img: texture_2d<f32>;
@group(2) @binding(1) var ismp: sampler;
struct SOut {
  @builtin(position) pos: vec4f,
  @location(0) a: vec2f,     // max(a, b) = |local| - (half - radius): small near every edge
  @location(1) b: vec2f,
  @location(2) uv: vec2f,
  @location(3) r: f32,
  @location(4) page: f32,
  @location(5) c: vec4f,
  @location(6) fade: f32,
  @location(7) k: vec3f,     // a glyph's ink, or the white a FLOW line's dashes go toward (both times DIM)
  @location(8) fl: vec2f,    // a FLOW line: the upper clamp, pixels per dash cycle
};
@vertex fn svs(a: VIn) -> SOut {
  var o: SOut;
  let org = a.i_pos.xy + D.offset.xy;
  let sm = D.mode.x;
  let vr = a.i_meta.w;
  let shape = f32(vr & 15u);
  let gly = sm > 0.5 && sm < 1.5 && a.i_meta.x > 9u && shape > 0.5;   // a glyph of shape 1..15
  let flow = sm > 1.5 && sm < 2.5 && (vr & 4u) != 0u;
  o.k = vec3f(0.0);
  o.fl = vec2f(0.0);
  var px: vec2f;
  if (sm > 2.5) {
    let tl = floor(org - 0.5 * a.i_size.xy + 0.5);
    let t = a.a_p.xy + 0.5;
    px = tl + t * a.i_size.xy;
    o.uv = mix(a.i_aux.xy, vec2f(a.i_aux.z, a.i_angle), t);
    o.a = vec2f(0.0);
    o.b = vec2f(0.0);
    o.r = 0.0;
    o.page = f32(a.i_meta.w);
  } else {
    var hs: vec2f;
    var r = 0.0;
    var ux = vec2f(1.0, 0.0);
    var c = org;
    if (sm < 0.5) {
      hs = 0.5 * abs(a.i_size.xy);
      r = a.i_aux.x;
    } else if (sm < 1.5) {
      hs = vec2f(abs(a.i_size.x));
      r = hs.x;
      if (gly) {
        r = 0.0;
        ux = vec2f(cos(a.i_angle), -sin(a.i_angle));   // turned counter-clockwise as seen (y is down)
      }
    } else {
      let d = a.i_aux.xy - a.i_pos.xy;
      let len = length(d);
      if (len > 1e-6) { ux = d / len; }
      c = org + 0.5 * d;
      hs = vec2f(0.5 * len, 0.5 * abs(a.i_size.x));
    }
    r = clamp(r, 0.0, min(hs.x, hs.y));
    let l = a.a_p.xy * 2.0 * (hs + 1.0);   // one pixel of margin for the antialiased edge
    px = c + ux * l.x + vec2f(-ux.y, ux.x) * l.y;
    let inner = hs - r;
    o.a = l - inner;
    o.b = -l - inner;
    o.r = r;
    o.uv = vec2f(0.0);
    o.page = 0.0;
    if (gly) {
      o.a = l;
      o.b = vec2f(0.0);
      o.r = hs.x;
      o.page = shape;
    } else if (flow) {
      var fN = 6.0;
      if (D.params[5].w > 0.0) { fN = D.params[5].w; }
      let ph = fract(D.params[5].z);
      o.uv = vec2f((0.5 + l.x / max(2.0 * hs.x, 1e-6)) * fN - ph, -ph);
      o.fl = vec2f(fN - ph, 2.0 * hs.x / fN);
      o.page = 16.0;
    }
  }
  var rgb = a.i_color.rgb;
  if (sm < 2.5) {
    let cm = a.i_meta.w >> 6u;
    if (cm == 1u) { rgb = mix(D.params[1].rgb, D.params[2].rgb, clamp(a.i_angle, 0.0, 1.0)); }
    else if (cm == 2u) { rgb = rgb * D.params[3].rgb; }
  }
  o.c = vec4f(rgb * D.params[0].rgb, a.i_color.a);
  if (gly) { o.k = hd_ink(rgb) * D.params[0].rgb; } else if (flow) { o.k = D.params[0].rgb; }
  o.fade = D.params[0].w;
  o.pos = vec4f(px.x / F.view.x * 2.0 - 1.0, 1.0 - px.y / F.view.y * 2.0, 0.0, 1.0);
  return o;
}
@fragment fn sfs(i: SOut) -> @location(0) vec4f {
  var cov: f32;
  if (D.mode.w > 1.5) {
    let im = textureSampleLevel(img, ismp, i.uv, 0.0);
    return vec4f(im.rgb * i.c.rgb, im.a * i.c.a * i.fade);
  }
  if (D.mode.w > 0.5) {
    if (i.page < 0.5) { cov = textureSampleLevel(t0, smp, i.uv, 0.0).r; }
    else if (i.page < 1.5) { cov = textureSampleLevel(t1, smp, i.uv, 0.0).r; }
    else if (i.page < 2.5) { cov = textureSampleLevel(t2, smp, i.uv, 0.0).r; }
    else { cov = textureSampleLevel(t3, smp, i.uv, 0.0).r; }
  } else if (i.page > 0.5 && i.page < 15.5) {
    // a glyph's shape, in its square of half size i.r pixels
    let g = hd_glyphCov(floor(i.page + 0.5), vec2f(i.a.x, -i.a.y) / i.r, i.r);
    return vec4f(mix(i.c.rgb, i.k, g.y), i.c.a * max(g.x, g.y) * i.fade);
  } else {
    let q = max(i.a, i.b);
    let d = length(max(q, vec2f(0.0))) + min(max(q.x, q.y), 0.0) - i.r;
    cov = clamp(0.5 - d, 0.0, 1.0);
    if (i.page > 15.5) {
      let fl = hd_flow(clamp(i.uv.x, i.uv.y, i.fl.x), clamp(1.5 / max(i.fl.y, 0.001), 0.02, 0.25));
      return vec4f(mix(i.c.rgb, i.k, fl.x), i.c.a * cov * i.fade * fl.y);
    }
  }
  return vec4f(i.c.rgb, i.c.a * cov * i.fade);
}
`;

// ---- unit meshes: interleaved position (3) and normal (3), 16-bit indices ----
// COPIES of draw.js's builders, byte for byte (tools/webgpu_parity.py compares the sources).

function sphereMesh(segs, stacks) {
  const v = [], ix = [];
  for (let i = 0; i <= stacks; i++) {
    const t = (Math.PI * i) / stacks, st = Math.sin(t), ct = Math.cos(t);
    for (let j = 0; j <= segs; j++) {
      const f = (2 * Math.PI * j) / segs;
      const x = st * Math.cos(f), y = st * Math.sin(f);
      v.push(x, y, ct, x, y, ct);
    }
  }
  const w = segs + 1;
  for (let i = 0; i < stacks; i++) {
    for (let j = 0; j < segs; j++) {
      const a = i * w + j, b = a + w, c = b + 1, d = a + 1;
      if (i > 0) ix.push(a, c, d);
      if (i < stacks - 1) ix.push(a, b, c);
    }
  }
  return { v, ix };
}

function tubeMesh(sides, caps) {
  const v = [], ix = [];
  for (let j = 0; j <= sides; j++) {
    const f = (2 * Math.PI * j) / sides, x = Math.cos(f), y = Math.sin(f);
    v.push(x, y, 0, x, y, 0, x, y, 1, x, y, 0);
  }
  for (let j = 0; j < sides; j++) {
    const a = 2 * j, b = a + 2, c = b + 1, d = a + 1;
    ix.push(a, b, c, a, c, d);
  }
  if (caps) {
    for (let z = 0; z <= 1; z++) {
      const base = v.length / 6, nz = z ? 1 : -1;
      v.push(0, 0, z, 0, 0, nz);
      for (let j = 0; j < sides; j++) {
        const f = (2 * Math.PI * j) / sides;
        v.push(Math.cos(f), Math.sin(f), z, 0, 0, nz);
      }
      for (let j = 0; j < sides; j++) {
        const p = base + 1 + j, q = base + 1 + ((j + 1) % sides);
        if (z) ix.push(base, p, q); else ix.push(base, q, p);
      }
    }
  }
  return { v, ix };
}

function capsuleMesh(sides, rings) {
  const prof = [];   // [radius, z, normal z]
  for (let k = rings; k >= 0; k--) { const f = (Math.PI / 2) * (k / rings); prof.push([Math.cos(f), -Math.sin(f), -Math.sin(f)]); }
  for (let k = 0; k <= rings; k++) { const f = (Math.PI / 2) * (k / rings); prof.push([Math.cos(f), 1 + Math.sin(f), Math.sin(f)]); }
  const v = [], ix = [], w = sides + 1;
  for (const [r, z, nz] of prof) {
    for (let j = 0; j <= sides; j++) {
      const f = (2 * Math.PI * j) / sides, x = Math.cos(f), y = Math.sin(f);
      v.push(x * r, y * r, z, x * r, y * r, nz);
    }
  }
  for (let i = 0; i + 1 < prof.length; i++) {
    for (let j = 0; j < sides; j++) {
      const a = i * w + j, b = a + 1, c = b + w, d = a + w;
      ix.push(a, b, c, a, c, d);
    }
  }
  return { v, ix };
}

function arcCapsuleMesh(sides, rings, segs) {
  const prof = [];
  for (let k = rings; k >= 0; k--) { const f = (Math.PI / 2) * (k / rings); prof.push([Math.cos(f), -Math.sin(f), -Math.sin(f)]); }
  for (let i = 1; i < segs; i++) prof.push([1, i / segs, 0]);
  for (let k = 0; k <= rings; k++) { const f = (Math.PI / 2) * (k / rings); prof.push([Math.cos(f), 1 + Math.sin(f), Math.sin(f)]); }
  const v = [], ix = [], w = sides + 1;
  for (const [r, z, nz] of prof) {
    for (let j = 0; j <= sides; j++) {
      const f = (2 * Math.PI * j) / sides, x = Math.cos(f), y = Math.sin(f);
      v.push(x * r, y * r, z, x * r, y * r, nz);
    }
  }
  for (let i = 0; i + 1 < prof.length; i++) {
    for (let j = 0; j < sides; j++) {
      const a = i * w + j, b = a + 1, c = b + w, d = a + w;
      ix.push(a, b, c, a, c, d);
    }
  }
  return { v, ix };
}

function arcRibbonMesh(arc, segs) {
  const v = [], ix = [];
  for (let i = 0; i <= segs; i++) {
    v.push(-1, 0, i / segs, 0, 1, 0, 1, 0, i / segs, 0, 1, 0);
    if (i > 0) { const a = 2 * (i - 1); ix.push(a, a + 1, a + 3, a, a + 3, a + 2); }
  }
  for (const end of [0, 1]) {
    const c = v.length / 6;
    v.push(0, 0, end, 0, 1, 0);
    for (let k = 0; k <= arc; k++) {
      const a = (Math.PI * k) / arc;
      v.push(Math.cos(a), 0, end ? 1 + Math.sin(a) : -Math.sin(a), 0, 1, 0);
      if (k > 0) ix.push(c, c + k, c + k + 1);
    }
  }
  return { v, ix };
}

function ribbonMesh(arc) {
  const v = [], ix = [];
  for (let k = 0; k <= arc; k++) { const a = (Math.PI * k) / arc; v.push(Math.cos(a), 0, -Math.sin(a), 0, 1, 0); }
  for (let k = 0; k <= arc; k++) { const a = (Math.PI * k) / arc; v.push(-Math.cos(a), 0, 1 + Math.sin(a), 0, 1, 0); }
  for (let k = 1; k + 1 < v.length / 6; k++) ix.push(0, k, k + 1);
  return { v, ix };
}

function boxMesh() {
  const faces = [
    [[1, 0, 0], [0, 1, 0], [0, 0, 1]], [[-1, 0, 0], [0, 0, 1], [0, 1, 0]],
    [[0, 1, 0], [0, 0, 1], [1, 0, 0]], [[0, -1, 0], [1, 0, 0], [0, 0, 1]],
    [[0, 0, 1], [1, 0, 0], [0, 1, 0]], [[0, 0, -1], [0, 1, 0], [1, 0, 0]],
  ];
  const corners = [[-1, -1], [1, -1], [1, 1], [-1, 1]];
  const v = [], ix = [];
  for (let f = 0; f < 6; f++) {
    const [n, u, w] = faces[f];
    for (const [a, b] of corners) {
      v.push(
        0.5 * (n[0] + a * u[0] + b * w[0]),
        0.5 * (n[1] + a * u[1] + b * w[1]),
        0.5 * (n[2] + a * u[2] + b * w[2]) + 0.5,
        n[0], n[1], n[2],
      );
    }
    const k = 4 * f;
    ix.push(k, k + 1, k + 2, k, k + 2, k + 3);
  }
  return { v, ix };
}

function quadMesh() {
  return {
    v: [-0.5, -0.5, 0, 0, 0, 1, 0.5, -0.5, 0, 0, 0, 1, 0.5, 0.5, 0, 0, 0, 1, -0.5, 0.5, 0, 0, 0, 1],
    ix: [0, 1, 2, 0, 2, 3],
  };
}

function discMesh(sides) {
  const v = [0, 0, 0, 0, 0, 1], ix = [];
  for (let j = 0; j < sides; j++) {
    const f = (2 * Math.PI * j) / sides;
    v.push(Math.cos(f), Math.sin(f), 0, 0, 0, 1);
    ix.push(0, 1 + j, 1 + ((j + 1) % sides));
  }
  return { v, ix };
}

// Slots as in draw.js: index = kind; 11 and 12 the coarse sphere and rod, 13 and 14 the flat line and arc.
const KIND_CODE = [0, 1, 3, 3, 4, 5, 5, 0, 0, 2, 6, 0, 0, 3, 3];
const SLOT_COARSE_SPHERE = 11, SLOT_COARSE_ROD = 12, SLOT_FLAT_LINE = 13, SLOT_FLAT_ARC = 14, N_SLOT = 15;

const MESH_LAYOUT = {
  arrayStride: 24, stepMode: 'vertex',
  attributes: [
    { shaderLocation: 0, offset: 0, format: 'float32x3' },
    { shaderLocation: 1, offset: 12, format: 'float32x3' },
  ],
};
// The record, as it is: only bytes 0..43 and the variant byte's word (48..51) are read.
const INSTANCE_LAYOUT = {
  arrayStride: INSTANCE_BYTES, stepMode: 'instance',
  attributes: [
    { shaderLocation: 2, offset: 0, format: 'float32x3' },
    { shaderLocation: 3, offset: 12, format: 'float32x3' },
    { shaderLocation: 4, offset: 24, format: 'float32x3' },
    { shaderLocation: 5, offset: 36, format: 'float32' },
    { shaderLocation: 6, offset: 40, format: 'unorm8x4' },
    { shaderLocation: 7, offset: 48, format: 'uint8x4' },
  ],
};

async function checkedModule(device, label, code) {
  const m = device.createShaderModule({ label, code });
  if (m.getCompilationInfo) {
    const info = await m.getCompilationInfo();
    const errs = info.messages.filter((x) => x.type === 'error');
    if (errs.length) {
      throw new Error('holon-draw webgpu: ' + label + ' did not compile: ' + errs.map((x) => x.lineNum + ':' + x.linePos + ' ' + x.message).join('; '));
    }
  }
  return m;
}

/**
 * @param {GPUDevice} device a device the HOST owns
 * @param {{format: GPUTextureFormat, text?: {texture: (page: number) => GPUTexture|null}, sampleCount?: number, depthFormat?: GPUTextureFormat, sphereSegments?: number, rodSides?: number}} options
 */
export async function createDrawerWebGPU(device, options) {
  options = options || {};
  if (!device) throw new Error('holon-draw webgpu: no device');
  const format = options.format;
  if (!format) throw new Error('holon-draw webgpu: options.format is required (the colour target\'s format)');
  const sampleCount = options.sampleCount === 4 ? 4 : 1;
  const depthFormat = options.depthFormat || 'depth24plus';
  const text = options.text || null;
  const U = GPUBufferUsage, T = GPUTextureUsage;
  const align = Math.max(256, device.limits.minUniformBufferOffsetAlignment | 0);
  const alignF = align / 4;

  // Shader modules: the world with and without shadows, the depth-only shadow pass, the screen.
  const [mWorld, mWorldS, mZ, mScreen] = await Promise.all([
    checkedModule(device, 'holon-draw world', COMMON + worldVS(false, false) + worldFS(false)),
    checkedModule(device, 'holon-draw world+shadow', COMMON + worldVS(true, false) + worldFS(true)),
    checkedModule(device, 'holon-draw shadow map', COMMON + worldVS(false, true) + ZFS),
    checkedModule(device, 'holon-draw screen', COMMON + SCREEN),
  ]);

  // Layouts.
  const bglU = device.createBindGroupLayout({
    label: 'holon-draw uniforms',
    entries: [
      { binding: 0, visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT, buffer: { type: 'uniform', hasDynamicOffset: true, minBindingSize: FRAME_FLOATS * 4 } },
      { binding: 1, visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT, buffer: { type: 'uniform', hasDynamicOffset: true, minBindingSize: DRAW_FLOATS * 4 } },
    ],
  });
  const bglShadow = device.createBindGroupLayout({
    label: 'holon-draw shadow map',
    entries: [{ binding: 0, visibility: GPUShaderStage.FRAGMENT, texture: { sampleType: 'depth' } }],
  });
  const bglText = device.createBindGroupLayout({
    label: 'holon-draw atlas',
    entries: [0, 1, 2, 3].map((b) => ({ binding: b, visibility: GPUShaderStage.FRAGMENT, texture: { sampleType: 'float' } }))
      .concat([{ binding: 4, visibility: GPUShaderStage.FRAGMENT, sampler: { type: 'filtering' } }]),
  });
  const plWorld = device.createPipelineLayout({ bindGroupLayouts: [bglU] });
  const plWorldS = device.createPipelineLayout({ bindGroupLayouts: [bglU, bglShadow] });
  const bglImage = device.createBindGroupLayout({
    label: 'holon-draw image',
    entries: [
      { binding: 0, visibility: GPUShaderStage.FRAGMENT, texture: { sampleType: 'float' } },
      { binding: 1, visibility: GPUShaderStage.FRAGMENT, sampler: { type: 'filtering' } },
    ],
  });
  const plScreen = device.createPipelineLayout({ bindGroupLayouts: [bglU, bglText, bglImage] });

  const BLEND = {
    color: { srcFactor: 'src-alpha', dstFactor: 'one-minus-src-alpha', operation: 'add' },
    alpha: { srcFactor: 'one', dstFactor: 'one-minus-src-alpha', operation: 'add' },
  };
  function worldDesc(shadow, cull, depthOn) {
    return {
      label: 'holon-draw world' + (shadow ? '+shadow' : '') + (cull ? ' cull' : '') + (depthOn ? '' : ' nodepth'),
      layout: shadow ? plWorldS : plWorld,
      vertex: { module: shadow ? mWorldS : mWorld, entryPoint: 'vs', buffers: [MESH_LAYOUT, INSTANCE_LAYOUT] },
      fragment: { module: shadow ? mWorldS : mWorld, entryPoint: 'fs', targets: [{ format, blend: BLEND }] },
      primitive: { topology: 'triangle-list', frontFace: 'ccw', cullMode: cull ? 'back' : 'none' },
      depthStencil: { format: depthFormat, depthWriteEnabled: depthOn, depthCompare: depthOn ? 'less-equal' : 'always' },
      multisample: { count: sampleCount },
    };
  }
  function zDesc(cull) {
    return {
      label: 'holon-draw shadow map' + (cull ? ' cull' : ''),
      layout: plWorld,
      vertex: { module: mZ, entryPoint: 'vs', buffers: [MESH_LAYOUT, INSTANCE_LAYOUT] },
      fragment: { module: mZ, entryPoint: 'zfs', targets: [] },
      primitive: { topology: 'triangle-list', frontFace: 'ccw', cullMode: cull ? 'front' : 'none' },
      depthStencil: { format: 'depth32float', depthWriteEnabled: true, depthCompare: 'less-equal' },
    };
  }
  // Pipelines by key; the four without shadows are made now, the shadow ones on first use.
  const pipes = new Map();
  const key = (shadow, cull, depthOn) => (shadow ? 4 : 0) | (cull ? 2 : 0) | (depthOn ? 1 : 0);
  const made = await Promise.all([[false, false], [false, true], [true, false], [true, true]].map(([c, d]) => device.createRenderPipelineAsync(worldDesc(false, c, d))));
  pipes.set(key(false, false, false), made[0]); pipes.set(key(false, false, true), made[1]);
  pipes.set(key(false, true, false), made[2]); pipes.set(key(false, true, true), made[3]);
  function worldPipe(shadow, cull, depthOn) {
    const k = key(shadow, cull, depthOn);
    let p = pipes.get(k);
    if (!p) { p = device.createRenderPipeline(worldDesc(shadow, cull, depthOn)); pipes.set(k, p); }
    return p;
  }
  let zPipes = null;
  const screenPipe = await device.createRenderPipelineAsync({
    label: 'holon-draw screen',
    layout: plScreen,
    vertex: { module: mScreen, entryPoint: 'svs', buffers: [MESH_LAYOUT, INSTANCE_LAYOUT] },
    fragment: { module: mScreen, entryPoint: 'sfs', targets: [{ format, blend: BLEND }] },
    primitive: { topology: 'triangle-list', cullMode: 'none' },
    depthStencil: { format: depthFormat, depthWriteEnabled: false, depthCompare: 'always' },
    multisample: { count: sampleCount },
  });

  // Meshes.
  const owned = [];   // buffers and textures destroy() frees
  function mesh(m) {
    const vf = new Float32Array(m.v);
    const vbo = device.createBuffer({ size: vf.byteLength, usage: U.VERTEX | U.COPY_DST });
    device.queue.writeBuffer(vbo, 0, vf);
    const ix = new Uint16Array(m.ix.length + (m.ix.length & 1));   // writeBuffer wants a multiple of 4 bytes
    ix.set(m.ix);
    const ibo = device.createBuffer({ size: ix.byteLength, usage: U.INDEX | U.COPY_DST });
    device.queue.writeBuffer(ibo, 0, ix);
    owned.push(vbo, ibo);
    return { vbo, ibo, n: m.ix.length };
  }
  const segs = Math.max(8, options.sphereSegments | 0 || 28);
  const sides = Math.max(6, options.rodSides | 0 || 20);
  const mSphere = mesh(sphereMesh(segs, Math.max(4, segs >> 1)));
  const mSphereCoarse = mesh(sphereMesh(10, 6));
  const mRod = mesh(tubeMesh(sides, true));
  const mRodCoarse = mesh(tubeMesh(8, true));
  const mLine = mesh(capsuleMesh(8, 3));
  const mRibbon = mesh(ribbonMesh(10));
  const mArc = mesh(arcCapsuleMesh(8, 3, 16));
  const mArcRibbon = mesh(arcRibbonMesh(10, 16));
  const mBox = mesh(boxMesh());
  const mQuad = mesh(quadMesh());
  const mDisc = mesh(discMesh(24));

  const slots = new Array(N_SLOT).fill(null);
  const slot = (i, kind, m, mx, my, lit, spec, rim, cull) => {
    slots[i] = { mesh: m, kind, mx, my, lit, spec, rim, cull, kc: KIND_CODE[i] };
  };
  slot(KIND.QUAD, KIND.QUAD, mQuad, 2, 0, 0, 0, 0, false);
  slot(KIND.PATCH, KIND.PATCH, mQuad, 2, 0, 0, 0, 0, false);
  slot(KIND.LINE, KIND.LINE, mLine, 1, 0.5, 0, 0, 0, true);
  slot(KIND.ARC, KIND.ARC, mArc, 3, 0.5, 0, 0, 0, true);
  slot(KIND.DISC, KIND.DISC, mDisc, 2, 1, 0, 0, 0, false);
  slot(KIND.GLYPH, KIND.GLYPH, mDisc, 2, 1, 0, 0, 0, false);
  slot(KIND.BOX, KIND.BOX, mBox, 2, 0, 1, 0.05, 0, true);
  slot(KIND.PRISM, KIND.PRISM, mBox, 2, 0, 1, 0.05, 0, true);
  slot(KIND.SPHERE, KIND.SPHERE, mSphere, 0, 0, 1, 0.3, 0.18, true);
  slot(KIND.ROD, KIND.ROD, mRod, 1, 1, 1, 0.18, 0.08, true);
  slot(SLOT_COARSE_SPHERE, KIND.SPHERE, mSphereCoarse, 0, 0, 1, 0.3, 0.18, true);
  slot(SLOT_COARSE_ROD, KIND.ROD, mRodCoarse, 1, 1, 1, 0.18, 0.08, true);
  slot(SLOT_FLAT_LINE, KIND.LINE, mRibbon, 1, 0.5, 0, 0, 0, false);
  slot(SLOT_FLAT_ARC, KIND.ARC, mArcRibbon, 3, 0.5, 0, 0, 0, false);

  const sslots = new Array(12).fill(null);
  sslots[KIND.QUAD] = { kind: KIND.QUAD, smode: 0 };
  sslots[KIND.PATCH] = { kind: KIND.PATCH, smode: 0 };
  sslots[KIND.DISC] = { kind: KIND.DISC, smode: 1 };
  sslots[KIND.GLYPH] = { kind: KIND.GLYPH, smode: 1 };
  sslots[KIND.LINE] = { kind: KIND.LINE, smode: 2 };
  sslots[KIND.ARC] = { kind: KIND.ARC, smode: 2 };
  if (text) sslots[KIND.RUN] = { kind: KIND.RUN, smode: 3 };
  sslots[KIND.IMAGE] = { kind: KIND.IMAGE, smode: 3, image: true };

  // The atlas bind group: the provider's pages, a 1x1 empty page where a page does not exist.
  const blank = device.createTexture({ size: [1, 1], format: 'r8unorm', usage: T.TEXTURE_BINDING });
  owned.push(blank);
  const blankView = blank.createView();
  const linear = device.createSampler({ magFilter: 'linear', minFilter: 'linear', addressModeU: 'clamp-to-edge', addressModeV: 'clamp-to-edge' });
  const views = new WeakMap();
  const viewOf = (t) => { let v = views.get(t); if (!v) { v = t.createView(); views.set(t, v); } return v; };
  const boundPages = [null, null, null, null];

  // Images, by slot: { tex, group } (setImage). A blank 1x1 group fills group 2 otherwise.
  const images = new Map();
  const nearestS = device.createSampler({ magFilter: 'nearest', minFilter: 'nearest', addressModeU: 'clamp-to-edge', addressModeV: 'clamp-to-edge' });
  const blankImg = device.createTexture({ size: [1, 1], format: 'rgba8unorm', usage: T.TEXTURE_BINDING });
  owned.push(blankImg);
  const blankImgBG = device.createBindGroup({ layout: bglImage, entries: [{ binding: 0, resource: blankImg.createView() }, { binding: 1, resource: linear }] });
  let textBG = null;
  function atlasGroup() {
    let same = !!textBG;
    for (let u = 0; u < MAX_PAGES; u++) {
      const t = text && sslots[KIND.RUN] ? text.texture(u) : null;
      if (t !== boundPages[u]) { boundPages[u] = t; same = false; }
    }
    if (same) return textBG;
    textBG = device.createBindGroup({
      layout: bglText,
      entries: [0, 1, 2, 3].map((u) => ({ binding: u, resource: boundPages[u] ? viewOf(boundPages[u]) : blankView }))
        .concat([{ binding: 4, resource: linear }]),
    });
    return textBG;
  }

  // One instance buffer per kind, grown as needed and reused. Within one draw() every batch
  // of a kind is written at its own offset in that buffer, as in draw.js.
  const instBuf = new Array(13).fill(null);
  const instCap = new Int32Array(13);
  const need = new Int32Array(13);
  const cursor = new Int32Array(13);

  // Uniforms: [main frame][shadow frame][one Draw per draw call], each slot `align` bytes.
  let ubuf = null, ucap = 0, uniBG = null;
  let ustage = new Float32Array(0);
  function ensureUniforms(nSlots) {
    const bytes = nSlots * align;
    if (bytes <= ucap) return;
    let cap = ucap || 64 * align;
    while (cap < bytes) cap *= 2;
    if (ubuf) ubuf.destroy();
    ubuf = device.createBuffer({ label: 'holon-draw uniforms', size: cap, usage: U.UNIFORM | U.COPY_DST });
    ucap = cap;
    const ns = new Float32Array(cap / 4);
    ustage = ns;
    uniBG = device.createBindGroup({
      layout: bglU,
      entries: [
        { binding: 0, resource: { buffer: ubuf, offset: 0, size: FRAME_FLOATS * 4 } },
        { binding: 1, resource: { buffer: ubuf, offset: 0, size: DRAW_FLOATS * 4 } },
      ],
    });
  }

  // The shadow map, made on first use.
  let zTex = null, zView = null, zSize = 0, zBG = null;
  function shadowTarget(size) {
    if (zTex && zSize === size) return;
    if (zTex) zTex.destroy();
    zTex = device.createTexture({ label: 'holon-draw shadow map', size: [size, size], format: 'depth32float', usage: T.RENDER_ATTACHMENT | T.TEXTURE_BINDING });
    zView = zTex.createView();
    zSize = size;
    zBG = device.createBindGroup({ layout: bglShadow, entries: [{ binding: 0, resource: zView }] });
  }

  // The drawer's own depth texture and (at 4x) its own colour texture, sized to the target.
  let dTex = null, dView = null, msTex = null, msView = null, ownW = 0, ownH = 0;
  function ownTargets(w, h, needDepth) {
    if (w !== ownW || h !== ownH) {
      if (dTex) { dTex.destroy(); dTex = null; dView = null; }
      if (msTex) { msTex.destroy(); msTex = null; msView = null; }
      ownW = w; ownH = h;
    }
    if (needDepth && !dTex) {
      dTex = device.createTexture({ label: 'holon-draw depth', size: [w, h], format: depthFormat, sampleCount, usage: T.RENDER_ATTACHMENT });
      dView = dTex.createView();
    }
    if (sampleCount > 1 && !msTex) {
      msTex = device.createTexture({ label: 'holon-draw 4x colour', size: [w, h], format, sampleCount, usage: T.RENDER_ATTACHMENT });
      msView = msTex.createView();
    }
  }

  // Scratch, allocated once.
  const light = new Float32Array(3);
  const lvp = new Float32Array(16);
  let wAt = new Int32Array(64);
  const inv = new Float64Array(16);
  const st = { drawCalls: 0, instances: 0, bytesUploaded: 0, paramUploads: 0, uniformBytes: 0, missingImages: 0, residentBuffers: 0, residentBytes: 0 };
  let dead = false;
  let flat = false;
  const lastP = new Float32Array(32);

  function eyeFrom(m) {
    const a00 = m[0], a01 = m[1], a02 = m[2], a03 = m[3], a10 = m[4], a11 = m[5], a12 = m[6], a13 = m[7];
    const a20 = m[8], a21 = m[9], a22 = m[10], a23 = m[11], a30 = m[12], a31 = m[13], a32 = m[14], a33 = m[15];
    const b00 = a00 * a11 - a01 * a10, b01 = a00 * a12 - a02 * a10, b02 = a00 * a13 - a03 * a10;
    const b03 = a01 * a12 - a02 * a11, b04 = a01 * a13 - a03 * a11, b05 = a02 * a13 - a03 * a12;
    const b06 = a20 * a31 - a21 * a30, b07 = a20 * a32 - a22 * a30, b08 = a20 * a33 - a23 * a30;
    const b09 = a21 * a32 - a22 * a31, b10 = a21 * a33 - a23 * a31, b11 = a22 * a33 - a23 * a32;
    const det = b00 * b11 - b01 * b10 + b02 * b09 + b03 * b08 - b04 * b07 + b05 * b06;
    if (!(Math.abs(det) > 0) || !isFinite(det)) {
      inv[0] = 0; inv[1] = 0; inv[2] = 1; inv[3] = 0;
      return;
    }
    let x = -(a31 * b05 - a32 * b04 + a33 * b03) / det;
    let y = -(a32 * b02 - a30 * b05 - a33 * b01) / det;
    let z = -(a30 * b04 - a31 * b02 + a33 * b00) / det;
    let w = -(a31 * b01 - a30 * b03 - a32 * b00) / det;
    let s = Math.max(Math.abs(x), Math.abs(y), Math.abs(z), Math.abs(w));
    if (!(s > 0) || !isFinite(s)) { x = 0; y = 0; z = 1; w = 0; s = 1; }
    if (Math.abs(w) > 1e-12 * s) s = w; else { w = 0; s = Math.hypot(x, y, z) || 1; }
    inv[0] = x / s; inv[1] = y / s; inv[2] = z / s; inv[3] = w / s;
  }

  function slotOf(b) {
    const k = b.kind | 0;
    if (k < 1 || k > 12) return null;
    if (b.screen === true) return sslots[k];
    if (k > 10) return null;
    if (flat) {
      if (k === KIND.LINE) return slots[SLOT_FLAT_LINE];
      if (k === KIND.ARC) return slots[SLOT_FLAT_ARC];
    }
    if (b.coarse === true) {
      if (k === KIND.SPHERE) return slots[SLOT_COARSE_SPHERE];
      if (k === KIND.ROD) return slots[SLOT_COARSE_ROD];
    }
    return slots[k];
  }

  function lightMatrix(c, r) {
    const lx = light[0], ly = light[1], lz = light[2];
    const ex = c[0] + lx * 2 * r, ey = c[1] + ly * 2 * r, ez = c[2] + lz * 2 * r;
    let ux = 0, uy = 0, uz = 1;
    if (Math.abs(lz) > 0.99) { ux = 1; uz = 0; }
    let sx = uy * lz - uz * ly, sy = uz * lx - ux * lz, sz = ux * ly - uy * lx;
    let sl = Math.hypot(sx, sy, sz) || 1; sx /= sl; sy /= sl; sz /= sl;
    const vx = ly * sz - lz * sy, vy = lz * sx - lx * sz, vz = lx * sy - ly * sx;
    const n = 0.01 * r, f = 4 * r, k = 1 / r, zz = -2 / (f - n), zw = -(f + n) / (f - n);
    lvp[0] = k * sx; lvp[1] = k * vx; lvp[2] = zz * lx; lvp[3] = 0;
    lvp[4] = k * sy; lvp[5] = k * vy; lvp[6] = zz * ly; lvp[7] = 0;
    lvp[8] = k * sz; lvp[9] = k * vz; lvp[10] = zz * lz; lvp[11] = 0;
    lvp[12] = -k * (sx * ex + sy * ey + sz * ez);
    lvp[13] = -k * (vx * ex + vy * ey + vz * ez);
    lvp[14] = -zz * (lx * ex + ly * ey + lz * ez) + zw;
    lvp[15] = 1;
  }

  // Upload one batch at its kind's cursor, as it is; its offset in that kind's buffer.
  function upload(b, k, len) {
    const at = cursor[k];
    cursor[k] = at + len;
    const bytes = b.bytes;
    device.queue.writeBuffer(instBuf[k], at, bytes.buffer, bytes.byteOffset, len);
    st.bytesUploaded += len;
    return at;
  }

  // ---- resident buffers (draw.js's RESIDENT BUFFERS) ----
  // handles[h]: { buf (GPUBuffer), cap (bytes allocated), size (bytes of records now) }
  const handles = [null];
  const freeHandles = [];
  function handleOf(h, what) {
    const r = typeof h === 'number' ? handles[h] : undefined;
    if (!r) throw new Error('holon-draw: ' + what + ': no resident buffer with handle ' + h);
    return r;
  }
  function recordBytes(bytes, what) {
    if (!bytes || typeof bytes.byteLength !== 'number' || bytes.byteLength % INSTANCE_BYTES !== 0) {
      throw new Error('holon-draw: ' + what + ' takes whole 52-byte records (an ArrayBufferView whose byteLength is a multiple of 52)');
    }
  }
  function residentBuffer(n) {
    // at least 4 bytes: an empty resident buffer is allowed, a zero-sized vertex buffer is not useful
    return device.createBuffer({ label: 'holon-draw resident', size: Math.max(4, n), usage: U.VERTEX | U.COPY_DST });
  }
  function write(buf, bytes) {
    if (bytes.byteLength > 0) device.queue.writeBuffer(buf, 0, bytes.buffer, bytes.byteOffset, bytes.byteLength);
  }
  function resident(bytes) {
    if (dead) throw new Error('holon-draw: resident() after destroy()');
    recordBytes(bytes, 'resident()');
    const r = { buf: residentBuffer(bytes.byteLength), cap: bytes.byteLength, size: bytes.byteLength };
    write(r.buf, bytes);
    const h = freeHandles.length ? freeHandles.pop() : handles.length;
    handles[h] = r;
    st.residentBuffers++;
    st.residentBytes += r.cap;
    return h;
  }
  function update(h, bytes) {
    if (dead) throw new Error('holon-draw: update() after destroy()');
    const r = handleOf(h, 'update()');
    recordBytes(bytes, 'update()');
    if (bytes.byteLength > r.cap) {
      r.buf.destroy();
      r.buf = residentBuffer(bytes.byteLength);
      st.residentBytes += bytes.byteLength - r.cap;
      r.cap = bytes.byteLength;
    }
    write(r.buf, bytes);
    r.size = bytes.byteLength;
  }
  function release(h) {
    const r = handleOf(h, 'release()');
    r.buf.destroy();
    handles[h] = null;
    freeHandles.push(h);
    st.residentBuffers--;
    st.residentBytes -= r.cap;
  }

  // Write a Draw slot; returns its byte offset. `cmpParams` counts what draw.js would upload.
  let uNext = 0;
  function drawSlot(params, o, mx, my, kc, sw, shade0, shade1, shade2, shade3) {
    const base = uNext * alignF;
    uNext++;
    const src = params || PARAMS_NEUTRAL;
    let same = true;
    for (let i = 0; i < 32; i++) {
      const v = Math.fround(src[i]);
      ustage[base + i] = v;
      if (lastP[i] !== v) { lastP[i] = v; same = false; }
    }
    if (!same) st.paramUploads++;
    ustage[base + 32] = o ? o[0] : 0; ustage[base + 33] = o ? o[1] : 0; ustage[base + 34] = o ? o[2] : 0; ustage[base + 35] = 0;
    ustage[base + 36] = mx; ustage[base + 37] = my; ustage[base + 38] = kc; ustage[base + 39] = sw;
    ustage[base + 40] = shade0; ustage[base + 41] = shade1; ustage[base + 42] = shade2; ustage[base + 43] = shade3;
    return base * 4;
  }
  // m: the frame's vp; m2: what F.lvp holds (the light's in the picture, the camera's in the shadow pass)
  function frameSlot(at, m, m2, l, eye, vw, vh, halfH, ppu, shp0, shp1, shp2, shp3, ox, oy) {
    const b = at / 4;
    for (let i = 0; i < 16; i++) ustage[b + i] = m[i];
    for (let i = 0; i < 16; i++) ustage[b + 16 + i] = m2[i];
    ustage[b + 32] = eye[0]; ustage[b + 33] = eye[1]; ustage[b + 34] = eye[2]; ustage[b + 35] = eye[3];
    ustage[b + 36] = l[0]; ustage[b + 37] = l[1]; ustage[b + 38] = l[2]; ustage[b + 39] = 0;
    ustage[b + 40] = vw; ustage[b + 41] = vh; ustage[b + 42] = halfH; ustage[b + 43] = ppu;
    ustage[b + 44] = shp0; ustage[b + 45] = shp1; ustage[b + 46] = shp2; ustage[b + 47] = shp3;
    ustage[b + 48] = ox; ustage[b + 49] = oy; ustage[b + 50] = 0; ustage[b + 51] = 0;
  }
  const eyeL = new Float64Array(4);
  const dynOff = [0, 0];

  /**
   * @param {Array} batches as draw.js takes them
   * @param {Float32Array} vp column-major, 16, GL clip convention (world batches only)
   * @param {object} opts draw.js's options and `target` (required), `size`, `depthTarget`, `clear`
   */
  function draw(batches, vp, opts) {
    if (dead) throw new Error('holon-draw webgpu: draw() after destroy()');
    if (!opts || !opts.target) throw new Error('holon-draw webgpu: opts.target (a GPUTexture or GPUTextureView) is required');
    st.drawCalls = 0; st.instances = 0; st.bytesUploaded = 0; st.paramUploads = 0; st.uniformBytes = 0; st.missingImages = 0;
    flat = opts.flatLines === true || (opts.flatLines !== false && opts.depthTest === false);
    const nb = batches.length;

    // Pass 1: bytes needed per kind; how many world and screen batches there are.
    need.fill(0);
    let nWorld = 0, nScreen = 0;
    for (let i = 0; i < nb; i++) {
      const b = batches[i], c = b.count | 0;
      if (c <= 0) continue;
      const s = slotOf(b);
      if (!s) continue;
      if (b.handle !== undefined) {
        const r = handleOf(b.handle, 'batch ' + i), first = b.first === undefined ? 0 : b.first;
        if (first !== (first | 0) || first < 0 || (first + c) * INSTANCE_BYTES > r.size) {
          throw new Error('holon-draw: batch ' + i + ' (kind ' + b.kind + ') asks for records ' + first + '..' + (first + c - 1) + ' of a resident buffer that holds ' + r.size / INSTANCE_BYTES);
        }
        if (s.image && !(b.slot === (b.slot | 0) && b.slot >= 0 && b.slot <= 255)) {
          throw new Error('holon-draw: batch ' + i + ' is a resident image batch: it needs `slot` (0..255), since its first record is not in hand');
        }
      } else {
        if (b.bytes.byteLength < c * INSTANCE_BYTES) {
          throw new Error('holon-draw: batch ' + i + ' (kind ' + b.kind + ') has ' + b.bytes.byteLength + ' bytes for ' + c + ' records of ' + INSTANCE_BYTES);
        }
        need[s.kind] += c * INSTANCE_BYTES;
      }
      if (b.screen === true) nScreen++; else nWorld++;
    }
    const clear = opts.clear;
    if (nWorld + nScreen === 0 && !clear) return;

    // The target, its size, the viewport.
    const target = opts.target;
    const isTex = typeof target.createView === 'function';
    const tview = isTex ? viewOf(target) : target;
    const V = opts.viewport;
    const tw = opts.size ? opts.size[0] | 0 : isTex ? target.width : V ? (V[0] + V[2]) | 0 : 0;
    const th = opts.size ? opts.size[1] | 0 : isTex ? target.height : V ? (V[1] + V[3]) | 0 : 0;
    if (!(tw > 0 && th > 0)) throw new Error('holon-draw webgpu: the target\'s size is unknown: pass a GPUTexture, or opts.size, or opts.viewport');
    const vw = V ? V[2] : tw, vh = V ? V[3] : th;
    ownTargets(tw, th, !opts.depthTarget);

    for (let k = 1; k <= 12; k++) {
      cursor[k] = 0;
      if (need[k] <= instCap[k]) continue;
      let cap = instCap[k] || 64 * INSTANCE_BYTES;
      while (cap < need[k]) cap *= 2;
      if (instBuf[k]) instBuf[k].destroy();
      instBuf[k] = device.createBuffer({ label: 'holon-draw instances ' + k, size: cap, usage: U.VERTEX | U.COPY_DST });
      instCap[k] = cap;
    }
    ensureUniforms(2 + 2 * nWorld + nScreen);
    uNext = 2;
    lastP.fill(NaN);

    const enc = device.createCommandEncoder({ label: 'holon-draw' });
    const so = opts.shadow;
    const shadowsOn = nWorld > 0 && !!(so && so.center && so.radius > 0);
    const ppu = opts.pixelsPerUnit > 0 ? +opts.pixelsPerUnit : 0;
    const flatLit = !!opts.flatLit;
    let ambient = typeof opts.ambient === 'number' ? opts.ambient : 0.25;
    ambient = ambient < 0 ? 0 : ambient > 1 ? 1 : ambient;
    const depthOn = opts.depthTest !== false;

    // Pass 2a: upload the world batches (read by the shadow pass too); the light; the eye.
    if (nWorld > 0) {
      if (wAt.length < nb) wAt = new Int32Array(Math.max(nb, 2 * wAt.length));
      for (let i = 0; i < nb; i++) {
        const b = batches[i], c = b.count | 0;
        const s = c > 0 && b.screen !== true ? slotOf(b) : null;
        wAt[i] = !s ? -1 : b.handle !== undefined ? -2 : upload(b, s.kind, c * INSTANCE_BYTES);   // -2: resident
      }
      let lx = 0.35, ly = -0.5, lz = 0.8;
      const L = opts.light;
      if (L) { lx = +L[0]; ly = +L[1]; lz = +L[2]; }
      let ll = Math.sqrt(lx * lx + ly * ly + lz * lz);
      if (!(ll > 0) || !isFinite(ll)) { lx = 0; ly = 0; lz = 1; ll = 1; }
      light[0] = lx / ll; light[1] = ly / ll; light[2] = lz / ll;
      eyeFrom(vp);
    }

    // The shadow map, drawn from the light, before the picture.
    if (shadowsOn) {
      if (!zPipes) zPipes = [device.createRenderPipeline(zDesc(false)), device.createRenderPipeline(zDesc(true))];
      const size = Math.max(64, Math.min(so.size | 0 || 1024, device.limits.maxTextureDimension2D));
      shadowTarget(size);
      lightMatrix(so.center, +so.radius);
      eyeL[0] = light[0]; eyeL[1] = light[1]; eyeL[2] = light[2]; eyeL[3] = 0;
      // F.lvp holds the camera's vp and F.view.xy its viewport's size: the ring is the camera's
      frameSlot(align, lvp, vp || lvp, light, eyeL, vw, vh, 0.5 * size, ppu, 0, 0, 0, 0, 0, 0);
      const zp = enc.beginRenderPass({
        label: 'holon-draw shadow map',
        colorAttachments: [],
        depthStencilAttachment: { view: zView, depthClearValue: 1, depthLoadOp: 'clear', depthStoreOp: 'store' },
      });
      let zcur = null, zpipe = null;
      dynOff[0] = align;
      for (let i = 0; i < nb; i++) {
        if (wAt[i] === -1) continue;
        const b = batches[i];
        if (b.castShadow === false) continue;
        const s = slotOf(b);
        if (!(s.lit || ((s.kind === KIND.LINE || s.kind === KIND.ARC) && !flat))) continue;
        const c = b.count | 0;
        if (s !== zcur) {
          zcur = s;
          const p = zPipes[s.cull ? 1 : 0];
          if (p !== zpipe) { zpipe = p; zp.setPipeline(p); }
          zp.setVertexBuffer(0, s.mesh.vbo);
          zp.setIndexBuffer(s.mesh.ibo, 'uint16');
        }
        if (wAt[i] === -2) zp.setVertexBuffer(1, handles[b.handle].buf, (b.first | 0) * INSTANCE_BYTES);
        else zp.setVertexBuffer(1, instBuf[s.kind], wAt[i]);
        dynOff[1] = drawSlot(b.params, b.offset, s.mx, s.my, s.kc, 0, 0, 0, 0, 0);
        zp.setBindGroup(0, uniBG, dynOff);
        zp.drawIndexed(s.mesh.n, c);
        st.drawCalls++; st.instances += c;
      }
      zp.end();
    }

    // The main frame: the world's matrix, eye and light; the screen's viewport.
    frameSlot(0, vp || lvp, lvp, light, inv, vw, vh, 0.5 * vh, ppu,
      shadowsOn ? (typeof so.strength === 'number' ? Math.max(0, Math.min(1, so.strength)) : 0.55) : 0,
      shadowsOn ? 1 / zSize : 0,
      shadowsOn ? (typeof so.bias === 'number' ? so.bias : 0.0006) : 0,
      shadowsOn ? 3 * +so.radius / zSize : 0,
      V ? V[0] : 0, V ? th - V[1] - V[3] : 0);

    const cc = clear || [0, 0, 0, 0];
    const colour = sampleCount > 1
      ? { view: msView, resolveTarget: tview, loadOp: clear ? 'clear' : 'load', storeOp: 'store', clearValue: { r: +cc[0], g: +cc[1], b: +cc[2], a: +cc[3] } }
      : { view: tview, loadOp: clear ? 'clear' : 'load', storeOp: 'store', clearValue: { r: +cc[0], g: +cc[1], b: +cc[2], a: +cc[3] } };
    const pass = enc.beginRenderPass({
      label: 'holon-draw',
      colorAttachments: [colour],
      depthStencilAttachment: { view: opts.depthTarget || dView, depthClearValue: 1, depthLoadOp: clear ? 'clear' : 'load', depthStoreOp: 'store' },
    });
    if (V) pass.setViewport(V[0], th - V[1] - V[3], V[2], V[3], 0, 1);
    dynOff[0] = 0;
    lastP.fill(NaN);

    // Pass 2b: the world batches, in the order given.
    if (nWorld > 0) {
      if (shadowsOn) pass.setBindGroup(1, zBG);
      let cur = null, curPipe = null;
      for (let i = 0; i < nb; i++) {
        if (wAt[i] === -1) continue;
        const b = batches[i], c = b.count | 0;
        const s = slotOf(b);
        const sh = b.shade;
        const spec = sh ? +sh[0] : s.spec, rim = sh ? +sh[1] : s.rim;
        if (s !== cur) {
          cur = s;
          const p = worldPipe(shadowsOn, s.cull, depthOn);
          if (p !== curPipe) { curPipe = p; pass.setPipeline(p); }
          pass.setVertexBuffer(0, s.mesh.vbo);
          pass.setIndexBuffer(s.mesh.ibo, 'uint16');
        }
        if (wAt[i] === -2) pass.setVertexBuffer(1, handles[b.handle].buf, (b.first | 0) * INSTANCE_BYTES);
        else pass.setVertexBuffer(1, instBuf[s.kind], wAt[i]);
        dynOff[1] = drawSlot(b.params, b.offset, s.mx, s.my, s.kc, 0, s.lit || flatLit ? 1 : 0, ambient, spec, rim);
        pass.setBindGroup(0, uniBG, dynOff);
        pass.drawIndexed(s.mesh.n, c);
        st.drawCalls++; st.instances += c;
      }
    }

    // Pass 3: the screen batches, over everything, in the order given.
    if (nScreen > 0) {
      lastP.fill(NaN);
      pass.setPipeline(screenPipe);
      pass.setVertexBuffer(0, mQuad.vbo);
      pass.setIndexBuffer(mQuad.ibo, 'uint16');
      let atlas = false, imgBG = null;
      for (let i = 0; i < nb; i++) {
        const b = batches[i], c = b.count | 0;
        if (c <= 0 || b.screen !== true) continue;
        const s = slotOf(b);
        if (!s) continue;
        const want = s.image ? (images.get(b.handle !== undefined ? b.slot : b.bytes[51]) || null) : null;
        if (s.image && !want) { st.missingImages++; continue; }
        if (!atlas) {
          // Once per draw(): the atlas pages, looked up at draw time, so a page the provider
          // made or refilled this frame is the one sampled.
          atlas = true;
          pass.setBindGroup(1, atlasGroup());
          pass.setBindGroup(2, blankImgBG);
          imgBG = blankImgBG;
        }
        if (want && want.group !== imgBG) {
          imgBG = want.group;
          pass.setBindGroup(2, imgBG);
        }
        if (b.handle !== undefined) pass.setVertexBuffer(1, handles[b.handle].buf, (b.first | 0) * INSTANCE_BYTES);
        else pass.setVertexBuffer(1, instBuf[s.kind], upload(b, s.kind, c * INSTANCE_BYTES));
        dynOff[1] = drawSlot(b.params, b.offset, s.smode, 0, 0, s.image ? 2 : s.smode === 3 ? 1 : 0, 0, 0, 0, 0);
        ustage[dynOff[1] / 4 + 34] = 0;   // a screen tile's offset is x, y only
        pass.setBindGroup(0, uniBG, dynOff);
        pass.drawIndexed(mQuad.n, c);
        st.drawCalls++; st.instances += c;
      }
    }
    pass.end();

    st.uniformBytes = uNext * align;
    device.queue.writeBuffer(ubuf, 0, ustage, 0, uNext * alignF);
    device.queue.submit([enc.finish()]);
  }

  /** The counts of the last draw(), and residentBuffers/residentBytes now (as draw.js). The SAME object every call. */
  function stats() {
    return st;
  }

  /** Put a texture in an image slot (0..255); see Images in the header. */
  function setImage(slot, source, o) {
    if (dead) throw new Error('holon-draw: setImage() after destroy()');
    slot = slot | 0;
    if (slot < 0 || slot > 255) throw new Error('holon-draw: an image slot is 0..255');
    const w = source.videoWidth || source.naturalWidth || source.width, h = source.videoHeight || source.naturalHeight || source.height;
    if (!(w > 0 && h > 0)) throw new Error('holon-draw: setImage() needs a source with a size');
    let img = images.get(slot);
    if (!img || img.w !== w || img.h !== h || img.nearest !== !!(o && o.nearest)) {
      if (img) img.tex.destroy();
      const tex = device.createTexture({ label: 'holon-draw image ' + slot, size: [w, h], format: 'rgba8unorm', usage: T.TEXTURE_BINDING | T.COPY_DST | T.RENDER_ATTACHMENT });
      const nearest = !!(o && o.nearest);
      img = { tex, w, h, nearest, group: device.createBindGroup({ layout: bglImage, entries: [{ binding: 0, resource: tex.createView() }, { binding: 1, resource: nearest ? nearestS : linear }] }) };
      images.set(slot, img);
    }
    device.queue.copyExternalImageToTexture({ source, flipY: false }, { texture: img.tex, premultipliedAlpha: false }, [w, h]);
  }

  /** Free an image slot's texture. */
  function deleteImage(slot) {
    const img = images.get(slot | 0);
    if (!img) return;
    img.tex.destroy();
    images.delete(slot | 0);
  }

  function destroy() {
    if (dead) return;
    for (let h = 1; h < handles.length; h++) if (handles[h]) release(h);
    dead = true;
    for (const img of images.values()) img.tex.destroy();
    images.clear();
    for (const x of owned) x.destroy();
    for (let k = 0; k < instBuf.length; k++) {
      if (instBuf[k]) instBuf[k].destroy();
      instBuf[k] = null;
      instCap[k] = 0;
    }
    if (ubuf) ubuf.destroy();
    if (zTex) zTex.destroy();
    if (dTex) dTex.destroy();
    if (msTex) msTex.destroy();
    ubuf = zTex = dTex = msTex = null;
  }

  return { draw, stats, destroy, setImage, deleteImage, resident, update, release, backend: 'webgpu', sampleCount, screenPrecision: 'highp' };
}
