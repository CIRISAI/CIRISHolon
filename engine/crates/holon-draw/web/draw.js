// holon-draw, the browser backend: draws the core's packed instance batches with instancing.
//
// One ES module, no dependency. The Rust core (`../src/lib.rs`) packs a scene into batches
// of fixed 52-byte records (`Instance::pack`); this file uploads each batch as it is and
// draws it with one instanced call. Nothing is built per vertex or per record in script.
//
// THE RECORD (little-endian, 52 bytes):
//   f32 pos[3] @0   f32 size[3] @12   f32 aux[3] @24   f32 angle @36
//   u8 rgba[4] @40 (straight alpha)   u32 tag @44 (not read here)
//   u8 kind @48   u8 detail @49   u8 flags @50 (bit 0 pinned, bit 1 coarse)   u8 variant @51
// Only bytes 0..43 reach a shader. Bytes 44..51 are uploaded and never pointed at.
//
// KINDS (world z is up):
//   1 quad, 9 patch   flat rectangle in the plane z = pos.z, size[0] by size[1], turned by angle about z
//   2 line, 3 arc     a thin 6-sided tube from pos to aux, WIDTH size[0] (an arc is its chord for now)
//   4 disc, 10 glyph  flat 24-gon in the plane z = pos.z, radius size[0]
//   5 box, 6 prism    base centre pos, size = (x, y, z extents), turned by angle about z (prism: flat top)
//   7 sphere          centre pos, radius size[0]
//   8 rod             capped cylinder from pos to aux, radius size[0]
// Boxes, prisms, spheres and rods are lit. Flat marks and lines are drawn in their exact
// colour unless `opts.flatLit` is set.
//
// SCREEN BATCHES (`batch.screen === true`): positions are DEVICE PIXELS from the viewport's
// top-left, y down; `offset` is the tile's origin in pixels; `vp` does not apply. They are
// drawn AFTER every world batch, in the order given, with the depth test off. Kinds there:
//   1 quad, 9 patch   rectangle size[0] by size[1] centred at pos, corner radius aux[0] px
//   4 disc, 10 glyph  filled circle, radius size[0] px
//   2 line, 3 arc     from pos to aux, width size[0] px, flat ends
//   11 run            a text run from the text provider's atlas (`./text.js`): rectangle
//                     size[0] by size[1] centred at pos, its top-left rounded to a whole pixel;
//                     u0 = aux[0], v0 = aux[1], u1 = aux[2], v1 = angle; atlas page = variant
//                     (byte 51, pages 0..3); colour = rgba.rgb, alpha = rgba.a * coverage.
// Shapes are cut by a signed distance in the fragment stage, with about 1 px of antialiasing.
// Other kinds in a screen batch, a run in a world batch, and a run with no provider bound
// are skipped. The viewport is `opts.viewport` ([x, y, w, h], as GL has it) or, without it,
// gl.getParameter(gl.VIEWPORT), which allocates: a host that draws screen batches every
// frame passes `opts.viewport`.
//
// PRECISION (INTERFACE.md §7): NO UNIFORM NAME APPEARS IN BOTH SHADER STAGES, in either
// program. World program: the vertex stage owns u_vp, u_offset, u_eye, u_mode; the fragment
// stage owns u_light, u_shade. Screen program: the vertex stage owns u_view, u_soffset,
// u_smode; the fragment stage owns u_fmode, u_t0..u_t3. So there is no pair whose precision
// could disagree at link. The world fragment stage is `mediump` throughout. The screen
// fragment stage wants more (pixel distances and atlas coordinates up to a few thousand),
// so its precision is a word, PF, written into BOTH stages' source by this file: `highp`
// where gl.getShaderPrecisionFormat says the fragment stage has it, else `mediump`. It
// never reads GL_FRAGMENT_PRECISION_HIGH; `options.forceMediump` is the no-high-precision
// test. Varyings are declared with the same explicit precision in both stages. On a device
// with mediump only, text sampling and large rounded shapes lose accuracy (not seen here:
// no such device in the test).
//
// GL STATE. draw() does not clear, does not set the viewport, does not touch the bound
// framebuffer, scissor, stencil or colour mask. It CHANGES, and leaves changed:
//   - the current program (left as one of this drawer's two programs);
//   - BLEND enabled, blendFuncSeparate(SRC_ALPHA, ONE_MINUS_SRC_ALPHA, ONE, ONE_MINUS_SRC_ALPHA)
//     (the blend equation is not touched: a host that uses another must restore FUNC_ADD first);
//   - DEPTH_TEST enabled, depthFunc(LEQUAL), depthMask(true) (the test is off while screen
//     batches are drawn and on again on return);
//   - CULL_FACE (left enabled), cullFace(BACK), frontFace(CCW);
//   - ARRAY_BUFFER binding (left null);
//   - the vertex array object binding, where one exists (WebGL 2 or OES_vertex_array_object):
//     left null, i.e. the default object, whose own attribute state is NOT touched;
//   - where no vertex array object exists (bare WebGL 1): attribute slots 0..7 (pointers,
//     enabled flags, divisors) and the ELEMENT_ARRAY_BUFFER binding. On return slots 0..7 are
//     disabled, their divisors are 0, and ELEMENT_ARRAY_BUFFER is null;
//   - only when a run batch is drawn: activeTexture (left TEXTURE0) and the TEXTURE_2D
//     bindings of units 0..3 (left on the atlas pages that exist; a unit whose page does
//     not exist is not touched).
// A draw() with nothing to draw changes no state.
//
// Translucent records are blended in the order given and write depth; a host that needs
// correct layering of translucent marks orders its batches back to front.
//
// A `vp` that mirrors the world (negative determinant) flips winding; closed meshes would
// then be culled from the wrong side. Not handled.

export const INSTANCE_BYTES = 52;

export const KIND = Object.freeze({
  QUAD: 1, LINE: 2, ARC: 3, DISC: 4, BOX: 5, PRISM: 6, SPHERE: 7, ROD: 8, PATCH: 9, GLYPH: 10, RUN: 11,
});

// Attribute slots, bound before link. Slot 0 is a mesh attribute (never instanced), which
// desktop GL needs.
// Slot 7 (the record's four trailing bytes) is read by the screen program only, which does
// not read slot 1: eight slots in all, WebGL 1's guaranteed minimum.
const A_P = 0, A_N = 1, I_POS = 2, I_SIZE = 3, I_AUX = 4, I_ANGLE = 5, I_COLOR = 6, I_META = 7;
const N_ATTR = 7;
const MAX_PAGES = 4;

const VS = `
precision highp float;
attribute vec3 a_p;
attribute vec3 a_n;
attribute vec3 i_pos;
attribute vec3 i_size;
attribute vec3 i_aux;
attribute float i_angle;
attribute vec4 i_color;
uniform mat4 u_vp;
uniform vec3 u_offset;
uniform vec4 u_eye;   // homogeneous eye: xyz - p*w points from p to the viewer
uniform vec2 u_mode;  // x: 0 sphere, 1 tube, 2 scaled-and-turned; y: radius scale
varying mediump vec3 v_n;
varying mediump vec3 v_v;
varying lowp vec4 v_c;
void main() {
  vec3 p;
  vec3 n;
  vec3 o = i_pos + u_offset;
  if (u_mode.x < 0.5) {
    n = a_n;
    p = o + a_p * i_size.x;
  } else if (u_mode.x < 1.5) {
    vec3 d = i_aux - i_pos;
    float len = length(d);
    vec3 dn = len > 1e-20 ? d / len : vec3(0.0, 0.0, 1.0);
    vec3 up = abs(dn.z) < 0.999 ? vec3(0.0, 0.0, 1.0) : vec3(1.0, 0.0, 0.0);
    vec3 bx = normalize(cross(up, dn));
    vec3 by = cross(dn, bx);
    float r = i_size.x * u_mode.y;
    p = o + (bx * a_p.x + by * a_p.y) * r + d * a_p.z;
    n = bx * a_n.x + by * a_n.y + dn * a_n.z;
  } else {
    vec3 s = u_mode.y > 0.5 ? i_size.xxx : i_size;
    float c = cos(i_angle);
    float sn = sin(i_angle);
    vec3 q = a_p * s;
    p = o + vec3(c * q.x - sn * q.y, sn * q.x + c * q.y, q.z);
    n = vec3(c * a_n.x - sn * a_n.y, sn * a_n.x + c * a_n.y, a_n.z);
  }
  v_n = n;
  v_v = normalize(u_eye.xyz - p * u_eye.w);
  v_c = i_color;
  gl_Position = u_vp * vec4(p, 1.0);
}
`;

const FS = `
precision mediump float;
uniform vec3 u_light;  // unit, toward the light
uniform vec4 u_shade;  // x lit (0 or 1), y ambient, z specular, w rim
varying mediump vec3 v_n;
varying mediump vec3 v_v;
varying lowp vec4 v_c;
void main() {
  vec3 n = normalize(v_n);
  if (!gl_FrontFacing) n = -n;
  vec3 v = normalize(v_v);
  float ndl = max(dot(n, u_light), 0.0);
  float sky = 0.75 + 0.25 * n.z;
  float lit = u_shade.y * sky + (1.0 - u_shade.y) * ndl;
  vec3 h = normalize(u_light + v);
  float sp = pow(max(dot(n, h), 0.0), 40.0) * u_shade.z * step(0.0001, ndl);
  float rim = pow(1.0 - max(dot(n, v), 0.0), 3.0) * u_shade.w;
  vec3 shaded = v_c.rgb * (lit + rim) + vec3(sp);
  gl_FragColor = vec4(mix(v_c.rgb, shaded, u_shade.x), v_c.a);
}
`;

// The screen program. PF is replaced by this file with one precision word, the same in both
// stages (see PRECISION above).
const SVS = `
precision highp float;
attribute vec3 a_p;
attribute vec3 i_pos;
attribute vec3 i_size;
attribute vec3 i_aux;
attribute float i_angle;
attribute vec4 i_color;
attribute vec4 i_meta;   // kind, detail, flags, variant as 0..255
uniform vec2 u_view;     // viewport width and height in pixels
uniform vec3 u_soffset;
uniform float u_smode;   // 0 rectangle, 1 circle, 2 line, 3 run
varying PF vec2 v_a;     // max(v_a, v_b) = |local| - (half - radius): small near every edge
varying PF vec2 v_b;
varying PF vec2 v_uv;
varying PF float v_r;
varying mediump float v_page;
varying lowp vec4 v_c;
void main() {
  vec2 o = i_pos.xy + u_soffset.xy;
  vec2 px;
  if (u_smode > 2.5) {
    vec2 tl = floor(o - 0.5 * i_size.xy + 0.5);
    vec2 t = a_p.xy + 0.5;
    px = tl + t * i_size.xy;
    v_uv = mix(i_aux.xy, vec2(i_aux.z, i_angle), t);
    v_a = vec2(0.0);
    v_b = vec2(0.0);
    v_r = 0.0;
    v_page = i_meta.w;
  } else {
    vec2 hs;
    float r = 0.0;
    vec2 ux = vec2(1.0, 0.0);
    vec2 c = o;
    if (u_smode < 0.5) {
      hs = 0.5 * abs(i_size.xy);
      r = i_aux.x;
    } else if (u_smode < 1.5) {
      hs = vec2(abs(i_size.x));
      r = hs.x;
    } else {
      vec2 d = i_aux.xy - i_pos.xy;
      float len = length(d);
      if (len > 1e-6) ux = d / len;
      c = o + 0.5 * d;
      hs = vec2(0.5 * len, 0.5 * abs(i_size.x));
    }
    r = clamp(r, 0.0, min(hs.x, hs.y));
    vec2 l = a_p.xy * 2.0 * (hs + 1.0);   // one pixel of margin for the antialiased edge
    px = c + ux * l.x + vec2(-ux.y, ux.x) * l.y;
    vec2 inner = hs - r;
    v_a = l - inner;
    v_b = -l - inner;
    v_r = r;
    v_uv = vec2(0.0);
    v_page = 0.0;
  }
  v_c = i_color;
  gl_Position = vec4(px.x / u_view.x * 2.0 - 1.0, 1.0 - px.y / u_view.y * 2.0, 0.0, 1.0);
}
`;

const SFS = `
precision PF float;
uniform sampler2D u_t0;
uniform sampler2D u_t1;
uniform sampler2D u_t2;
uniform sampler2D u_t3;
uniform mediump float u_fmode;   // 0 shape, 1 run
varying PF vec2 v_a;
varying PF vec2 v_b;
varying PF vec2 v_uv;
varying PF float v_r;
varying mediump float v_page;
varying lowp vec4 v_c;
void main() {
  float cov;
  if (u_fmode > 0.5) {
    if (v_page < 0.5) cov = texture2D(u_t0, v_uv).a;
    else if (v_page < 1.5) cov = texture2D(u_t1, v_uv).a;
    else if (v_page < 2.5) cov = texture2D(u_t2, v_uv).a;
    else cov = texture2D(u_t3, v_uv).a;
  } else {
    vec2 q = max(v_a, v_b);
    float d = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - v_r;
    cov = clamp(0.5 - d, 0.0, 1.0);
  }
  gl_FragColor = vec4(v_c.rgb, v_c.a * cov);
}
`;

// ---- unit meshes: interleaved position (3) and normal (3), 16-bit indices ----

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

// A tube along +z from 0 to 1, radius 1.
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

// x and y in [-1/2, 1/2], z in [0, 1].
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

// ---- slots: how a kind is drawn. Index = kind; 11 and 12 are the coarse sphere and rod. ----
// mode: [vertex mode, radius scale or uniform-scale switch]; shade: [lit, specular, rim]; cull.
const SLOT_COARSE_SPHERE = 11, SLOT_COARSE_ROD = 12, N_SLOT = 13;

function compile(gl, type, src) {
  const s = gl.createShader(type);
  gl.shaderSource(s, src);
  gl.compileShader(s);
  if (!gl.getShaderParameter(s, gl.COMPILE_STATUS)) {
    const log = gl.getShaderInfoLog(s);
    gl.deleteShader(s);
    throw new Error('holon-draw: shader did not compile: ' + log);
  }
  return s;
}

/**
 * @param {WebGLRenderingContext|WebGL2RenderingContext} gl a context the HOST owns
 * @param {{sphereSegments?: number, rodSides?: number, text?: {texture: (page: number) => WebGLTexture|null}, forceMediump?: boolean}} [options]
 *   text: a provider from `./text.js`; without one, run batches are skipped.
 *   forceMediump: write `mediump` for the screen program's fragment precision whatever the device has.
 */
export function createDrawer(gl, options) {
  options = options || {};
  const isGL2 = typeof WebGL2RenderingContext !== 'undefined' && gl instanceof WebGL2RenderingContext;

  // Instancing: native on WebGL 2, ANGLE_instanced_arrays on WebGL 1.
  let divisor, drawInstanced;
  if (isGL2) {
    divisor = (i, d) => gl.vertexAttribDivisor(i, d);
    drawInstanced = (n, count) => gl.drawElementsInstanced(gl.TRIANGLES, n, gl.UNSIGNED_SHORT, 0, count);
  } else {
    const ext = gl.getExtension('ANGLE_instanced_arrays');
    if (!ext) {
      throw new Error('holon-draw: this context has no instancing (neither WebGL 2 nor ANGLE_instanced_arrays on WebGL 1); nothing can be drawn');
    }
    divisor = (i, d) => ext.vertexAttribDivisorANGLE(i, d);
    drawInstanced = (n, count) => ext.drawElementsInstancedANGLE(gl.TRIANGLES, n, gl.UNSIGNED_SHORT, 0, count);
  }

  // Vertex array objects keep this drawer's attribute state off the host's.
  let createVAO = null, bindVAO = null, deleteVAO = null, boundVAO = null;
  if (isGL2) {
    boundVAO = () => gl.getParameter(gl.VERTEX_ARRAY_BINDING);
    createVAO = () => gl.createVertexArray();
    bindVAO = (v) => gl.bindVertexArray(v);
    deleteVAO = (v) => gl.deleteVertexArray(v);
  } else {
    const ext = gl.getExtension('OES_vertex_array_object');
    if (ext) {
      boundVAO = () => gl.getParameter(ext.VERTEX_ARRAY_BINDING_OES);
      createVAO = () => ext.createVertexArrayOES();
      bindVAO = (v) => ext.bindVertexArrayOES(v);
      deleteVAO = (v) => ext.deleteVertexArrayOES(v);
    }
  }
  const hasVAO = !!createVAO;

  // Program.
  const vs = compile(gl, gl.VERTEX_SHADER, VS);
  const fs = compile(gl, gl.FRAGMENT_SHADER, FS);
  const program = gl.createProgram();
  gl.attachShader(program, vs);
  gl.attachShader(program, fs);
  gl.bindAttribLocation(program, A_P, 'a_p');
  gl.bindAttribLocation(program, A_N, 'a_n');
  gl.bindAttribLocation(program, I_POS, 'i_pos');
  gl.bindAttribLocation(program, I_SIZE, 'i_size');
  gl.bindAttribLocation(program, I_AUX, 'i_aux');
  gl.bindAttribLocation(program, I_ANGLE, 'i_angle');
  gl.bindAttribLocation(program, I_COLOR, 'i_color');
  gl.linkProgram(program);
  gl.deleteShader(vs);
  gl.deleteShader(fs);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    const log = gl.getProgramInfoLog(program);
    gl.deleteProgram(program);
    throw new Error('holon-draw: program did not link: ' + log);
  }
  const uVP = gl.getUniformLocation(program, 'u_vp');
  const uOffset = gl.getUniformLocation(program, 'u_offset');
  const uEye = gl.getUniformLocation(program, 'u_eye');
  const uMode = gl.getUniformLocation(program, 'u_mode');
  const uLight = gl.getUniformLocation(program, 'u_light');
  const uShade = gl.getUniformLocation(program, 'u_shade');

  // The screen program.
  const hp = gl.getShaderPrecisionFormat(gl.FRAGMENT_SHADER, gl.HIGH_FLOAT);
  const PF = !options.forceMediump && hp && hp.precision > 0 ? 'highp' : 'mediump';
  const svs = compile(gl, gl.VERTEX_SHADER, SVS.replace(/\bPF\b/g, PF));
  const sfs = compile(gl, gl.FRAGMENT_SHADER, SFS.replace(/\bPF\b/g, PF));
  const sprogram = gl.createProgram();
  gl.attachShader(sprogram, svs);
  gl.attachShader(sprogram, sfs);
  gl.bindAttribLocation(sprogram, A_P, 'a_p');
  gl.bindAttribLocation(sprogram, I_POS, 'i_pos');
  gl.bindAttribLocation(sprogram, I_SIZE, 'i_size');
  gl.bindAttribLocation(sprogram, I_AUX, 'i_aux');
  gl.bindAttribLocation(sprogram, I_ANGLE, 'i_angle');
  gl.bindAttribLocation(sprogram, I_COLOR, 'i_color');
  gl.bindAttribLocation(sprogram, I_META, 'i_meta');
  gl.linkProgram(sprogram);
  gl.deleteShader(svs);
  gl.deleteShader(sfs);
  if (!gl.getProgramParameter(sprogram, gl.LINK_STATUS)) {
    const log = gl.getProgramInfoLog(sprogram);
    gl.deleteProgram(sprogram);
    gl.deleteProgram(program);
    throw new Error('holon-draw: screen program did not link: ' + log);
  }
  const uView = gl.getUniformLocation(sprogram, 'u_view');
  const uSOffset = gl.getUniformLocation(sprogram, 'u_soffset');
  const uSMode = gl.getUniformLocation(sprogram, 'u_smode');
  const uFMode = gl.getUniformLocation(sprogram, 'u_fmode');
  const uTex = [0, 1, 2, 3].map((i) => gl.getUniformLocation(sprogram, 'u_t' + i));
  let samplersSet = false;
  const text = options.text || null;

  // Meshes, built once. createDrawer() puts back the buffer and vertex array bindings it
  // found, and changes no other state.
  const prevArray = gl.getParameter(gl.ARRAY_BUFFER_BINDING);
  const prevElement = hasVAO ? null : gl.getParameter(gl.ELEMENT_ARRAY_BUFFER_BINDING);
  const meshes = [];
  function mesh(m) {
    const vbo = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(m.v), gl.STATIC_DRAW);
    const out = { vbo, ibo: gl.createBuffer(), n: m.ix.length, ix: new Uint16Array(m.ix) };
    meshes.push(out);
    return out;
  }
  const segs = Math.max(8, options.sphereSegments | 0 || 28);
  const sides = Math.max(6, options.rodSides | 0 || 20);
  const mSphere = mesh(sphereMesh(segs, Math.max(4, segs >> 1)));
  const mSphereCoarse = mesh(sphereMesh(10, 6));
  const mRod = mesh(tubeMesh(sides, true));
  const mRodCoarse = mesh(tubeMesh(8, true));
  const mLine = mesh(tubeMesh(6, false));
  const mBox = mesh(boxMesh());
  const mQuad = mesh(quadMesh());
  const mDisc = mesh(discMesh(24));

  // slot -> { mesh, kind (whose instance buffer), mode x, mode y, lit, spec, rim, cull, vao }
  const slots = new Array(N_SLOT).fill(null);
  const slot = (i, kind, m, mx, my, lit, spec, rim, cull) => {
    slots[i] = { mesh: m, kind, mx, my, lit, spec, rim, cull, vao: null };
  };
  slot(KIND.QUAD, KIND.QUAD, mQuad, 2, 0, 0, 0, 0, false);
  slot(KIND.PATCH, KIND.PATCH, mQuad, 2, 0, 0, 0, 0, false);
  slot(KIND.LINE, KIND.LINE, mLine, 1, 0.5, 0, 0, 0, true);
  slot(KIND.ARC, KIND.ARC, mLine, 1, 0.5, 0, 0, 0, true);
  slot(KIND.DISC, KIND.DISC, mDisc, 2, 1, 0, 0, 0, false);
  slot(KIND.GLYPH, KIND.GLYPH, mDisc, 2, 1, 0, 0, 0, false);
  slot(KIND.BOX, KIND.BOX, mBox, 2, 0, 1, 0.05, 0, true);
  slot(KIND.PRISM, KIND.PRISM, mBox, 2, 0, 1, 0.05, 0, true);
  slot(KIND.SPHERE, KIND.SPHERE, mSphere, 0, 0, 1, 0.3, 0.18, true);
  slot(KIND.ROD, KIND.ROD, mRod, 1, 1, 1, 0.18, 0.08, true);
  slot(SLOT_COARSE_SPHERE, KIND.SPHERE, mSphereCoarse, 0, 0, 1, 0.3, 0.18, true);
  slot(SLOT_COARSE_ROD, KIND.ROD, mRodCoarse, 1, 1, 1, 0.18, 0.08, true);

  // Screen slots, by kind: vertex mode (0 rectangle, 1 circle, 2 line, 3 run).
  const sslots = new Array(12).fill(null);
  sslots[KIND.QUAD] = { kind: KIND.QUAD, smode: 0 };
  sslots[KIND.PATCH] = { kind: KIND.PATCH, smode: 0 };
  sslots[KIND.DISC] = { kind: KIND.DISC, smode: 1 };
  sslots[KIND.GLYPH] = { kind: KIND.GLYPH, smode: 1 };
  sslots[KIND.LINE] = { kind: KIND.LINE, smode: 2 };
  sslots[KIND.ARC] = { kind: KIND.ARC, smode: 2 };
  if (text) sslots[KIND.RUN] = { kind: KIND.RUN, smode: 3 };
  let screenVAO = null;

  function meshAttribs(m) {
    gl.bindBuffer(gl.ARRAY_BUFFER, m.vbo);
    gl.vertexAttribPointer(A_P, 3, gl.FLOAT, false, 24, 0);
    gl.vertexAttribPointer(A_N, 3, gl.FLOAT, false, 24, 12);
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, m.ibo);
    for (let i = 0; i < N_ATTR; i++) {
      gl.enableVertexAttribArray(i);
      divisor(i, i >= I_POS ? 1 : 0);
    }
  }

  if (hasVAO) {
    const prevVAO = boundVAO();
    for (const s of slots) {
      if (!s) continue;
      s.vao = createVAO();
      bindVAO(s.vao);
      meshAttribs(s.mesh);
      gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, s.mesh.ix, gl.STATIC_DRAW);
    }
    screenVAO = createVAO();
    bindVAO(screenVAO);
    meshAttribs(mQuad);
    gl.enableVertexAttribArray(I_META);
    divisor(I_META, 1);
    bindVAO(prevVAO);
  } else {
    for (const m of meshes) {
      gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, m.ibo);
      gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, m.ix, gl.STATIC_DRAW);
    }
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, prevElement);
  }
  gl.bindBuffer(gl.ARRAY_BUFFER, prevArray);

  // One instance buffer per kind, grown as needed and reused. Within one draw() every batch
  // of a kind is written at its own offset in that buffer, so a later batch never overwrites
  // bytes an earlier draw call of the same frame still reads.
  const instBuf = new Array(12).fill(null);
  const instCap = new Int32Array(12);
  const need = new Int32Array(12);
  const cursor = new Int32Array(12);

  // Scratch, allocated once.
  const light = new Float32Array(3);
  const inv = new Float64Array(16);
  const st = { drawCalls: 0, instances: 0, bytesUploaded: 0 };
  let dead = false;

  // The eye, homogeneous, from vp alone: the preimage of clip (0, 0, -1, 0). For a
  // perspective vp that is the camera position (w != 0); for an orthographic one it is the
  // direction toward the viewer (w = 0).
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
    // Third column of the inverse (elements 8..11 of a column-major inverse), negated.
    let x = -(a31 * b05 - a32 * b04 + a33 * b03) / det;
    let y = -(a32 * b02 - a30 * b05 - a33 * b01) / det;
    let z = -(a30 * b04 - a31 * b02 + a33 * b00) / det;
    let w = -(a31 * b01 - a30 * b03 - a32 * b00) / det;
    // Scale so the numbers are tame in f32, and so that w >= 0.
    let s = Math.max(Math.abs(x), Math.abs(y), Math.abs(z), Math.abs(w));
    if (!(s > 0) || !isFinite(s)) { x = 0; y = 0; z = 1; w = 0; s = 1; }
    if (Math.abs(w) > 1e-12 * s) s = w; else { w = 0; s = Math.hypot(x, y, z) || 1; }
    inv[0] = x / s; inv[1] = y / s; inv[2] = z / s; inv[3] = w / s;
  }

  // The slot a batch is drawn with, or null if it is not drawn.
  function slotOf(b) {
    const k = b.kind | 0;
    if (k < 1 || k > 11) return null;
    if (b.screen === true) return sslots[k];
    if (k > 10) return null;
    if (b.coarse === true) {
      if (k === KIND.SPHERE) return slots[SLOT_COARSE_SPHERE];
      if (k === KIND.ROD) return slots[SLOT_COARSE_ROD];
    }
    return slots[k];
  }

  // Upload one batch at its kind's cursor and point the instanced attributes at it.
  function feed(b, k, len, meta) {
    const at = cursor[k];
    cursor[k] = at + len;
    gl.bindBuffer(gl.ARRAY_BUFFER, instBuf[k]);
    const bytes = b.bytes;
    if (bytes.byteLength === len) gl.bufferSubData(gl.ARRAY_BUFFER, at, bytes);
    else if (isGL2) gl.bufferSubData(gl.ARRAY_BUFFER, at, bytes, 0, len);
    else gl.bufferSubData(gl.ARRAY_BUFFER, at, bytes.subarray(0, len)); // the one view made; pass exact-length bytes to avoid it
    gl.vertexAttribPointer(I_POS, 3, gl.FLOAT, false, INSTANCE_BYTES, at);
    gl.vertexAttribPointer(I_SIZE, 3, gl.FLOAT, false, INSTANCE_BYTES, at + 12);
    gl.vertexAttribPointer(I_AUX, 3, gl.FLOAT, false, INSTANCE_BYTES, at + 24);
    gl.vertexAttribPointer(I_ANGLE, 1, gl.FLOAT, false, INSTANCE_BYTES, at + 36);
    gl.vertexAttribPointer(I_COLOR, 4, gl.UNSIGNED_BYTE, true, INSTANCE_BYTES, at + 40);
    if (meta) gl.vertexAttribPointer(I_META, 4, gl.UNSIGNED_BYTE, false, INSTANCE_BYTES, at + 48);
    st.drawCalls++;
    st.instances += b.count | 0;
    st.bytesUploaded += len;
  }

  /**
   * @param {Array<{kind:number, offset?:ArrayLike<number>, count:number, bytes:Uint8Array, screen?:boolean, coarse?:boolean}>} batches
   * @param {Float32Array} vp column-major, 16 (world batches only)
   * @param {{light?:ArrayLike<number>, ambient?:number, flatLit?:boolean, viewport?:ArrayLike<number>}} [opts]
   */
  function draw(batches, vp, opts) {
    if (dead) throw new Error('holon-draw: draw() after destroy()');
    st.drawCalls = 0; st.instances = 0; st.bytesUploaded = 0;
    const nb = batches.length;

    // Pass 1: bytes needed per kind; how many world and screen batches there are.
    need.fill(0);
    let nWorld = 0, nScreen = 0;
    for (let i = 0; i < nb; i++) {
      const b = batches[i], c = b.count | 0;
      if (c <= 0) continue;
      const s = slotOf(b);
      if (!s) continue;
      if (b.bytes.byteLength < c * INSTANCE_BYTES) {
        throw new Error('holon-draw: batch ' + i + ' (kind ' + b.kind + ') has ' + b.bytes.byteLength + ' bytes for ' + c + ' records of ' + INSTANCE_BYTES);
      }
      need[s.kind] += c * INSTANCE_BYTES;
      if (b.screen === true) nScreen++; else nWorld++;
    }
    if (nWorld + nScreen === 0) return;

    for (let k = 1; k <= 11; k++) {
      cursor[k] = 0;
      if (need[k] <= instCap[k]) continue;
      let cap = instCap[k] || 64 * INSTANCE_BYTES;
      while (cap < need[k]) cap *= 2;
      if (!instBuf[k]) instBuf[k] = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, instBuf[k]);
      gl.bufferData(gl.ARRAY_BUFFER, cap, gl.DYNAMIC_DRAW);
      instCap[k] = cap;
    }

    // Frame state.
    gl.enable(gl.BLEND);
    gl.blendFuncSeparate(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA, gl.ONE, gl.ONE_MINUS_SRC_ALPHA);
    gl.enable(gl.DEPTH_TEST);
    gl.depthFunc(gl.LEQUAL);
    gl.depthMask(true);
    gl.cullFace(gl.BACK);
    gl.frontFace(gl.CCW);
    gl.enable(gl.CULL_FACE);
    let culling = true;

    // Pass 2: the world batches, in the order given.
    if (nWorld > 0) {
      gl.useProgram(program);
      gl.uniformMatrix4fv(uVP, false, vp);
      eyeFrom(vp);
      gl.uniform4f(uEye, inv[0], inv[1], inv[2], inv[3]);
      let lx = 0.35, ly = -0.5, lz = 0.8;
      const L = opts && opts.light;
      if (L) { lx = +L[0]; ly = +L[1]; lz = +L[2]; }
      let ll = Math.sqrt(lx * lx + ly * ly + lz * lz);
      if (!(ll > 0) || !isFinite(ll)) { lx = 0; ly = 0; lz = 1; ll = 1; }
      light[0] = lx / ll; light[1] = ly / ll; light[2] = lz / ll;
      gl.uniform3fv(uLight, light);
      let ambient = opts && typeof opts.ambient === 'number' ? opts.ambient : 0.25;
      ambient = ambient < 0 ? 0 : ambient > 1 ? 1 : ambient;
      const flatLit = !!(opts && opts.flatLit);

      let cur = null;
      for (let i = 0; i < nb; i++) {
        const b = batches[i], c = b.count | 0;
        if (c <= 0 || b.screen === true) continue;
        const s = slotOf(b);
        if (!s) continue;
        if (s !== cur) {
          cur = s;
          if (hasVAO) bindVAO(s.vao); else meshAttribs(s.mesh);
          gl.uniform2f(uMode, s.mx, s.my);
          gl.uniform4f(uShade, s.lit || flatLit ? 1 : 0, ambient, s.spec, s.rim);
          if (s.cull !== culling) {
            culling = s.cull;
            if (culling) gl.enable(gl.CULL_FACE); else gl.disable(gl.CULL_FACE);
          }
        }
        feed(b, s.kind, c * INSTANCE_BYTES, false);
        const o = b.offset;
        if (o) gl.uniform3f(uOffset, o[0], o[1], o[2]); else gl.uniform3f(uOffset, 0, 0, 0);
        drawInstanced(s.mesh.n, c);
      }
    }

    // Pass 3: the screen batches, over everything, in the order given.
    if (nScreen > 0) {
      gl.useProgram(sprogram);
      gl.disable(gl.DEPTH_TEST);
      if (culling) { gl.disable(gl.CULL_FACE); culling = false; }
      const V = (opts && opts.viewport) || gl.getParameter(gl.VIEWPORT);
      gl.uniform2f(uView, V[2], V[3]);
      if (hasVAO) {
        bindVAO(screenVAO);
      } else {
        meshAttribs(mQuad);
        gl.enableVertexAttribArray(I_META);
        divisor(I_META, 1);
      }
      let smode = -1, textured = false;
      for (let i = 0; i < nb; i++) {
        const b = batches[i], c = b.count | 0;
        if (c <= 0 || b.screen !== true) continue;
        const s = slotOf(b);
        if (!s) continue;
        if (s.smode !== smode) {
          smode = s.smode;
          gl.uniform1f(uSMode, smode);
          gl.uniform1f(uFMode, smode === 3 ? 1 : 0);
        }
        if (smode === 3 && !textured) {
          // Once per draw(): the atlas pages on units 0..3. Pages are bound here, at draw
          // time, so a page the provider made or refilled this frame is the one sampled.
          textured = true;
          if (!samplersSet) {
            samplersSet = true;
            for (let u = 0; u < MAX_PAGES; u++) gl.uniform1i(uTex[u], u);
          }
          for (let u = MAX_PAGES - 1; u >= 0; u--) {
            const t = text.texture(u);
            if (!t) continue;
            gl.activeTexture(gl.TEXTURE0 + u);
            gl.bindTexture(gl.TEXTURE_2D, t);
          }
          gl.activeTexture(gl.TEXTURE0);
        }
        feed(b, s.kind, c * INSTANCE_BYTES, true);
        const o = b.offset;
        if (o) gl.uniform3f(uSOffset, o[0], o[1], 0); else gl.uniform3f(uSOffset, 0, 0, 0);
        drawInstanced(mQuad.n, c);
      }
      gl.enable(gl.DEPTH_TEST);
    }

    // Leave as documented at the top of this file.
    if (hasVAO) {
      bindVAO(null);
    } else {
      for (let i = 0; i <= I_META; i++) {
        divisor(i, 0);
        gl.disableVertexAttribArray(i);
      }
      gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, null);
    }
    gl.bindBuffer(gl.ARRAY_BUFFER, null);
    if (!culling) gl.enable(gl.CULL_FACE);
  }

  /** The counts of the last draw(). The SAME object every call (no allocation); copy it to keep it. */
  function stats() {
    return st;
  }

  function destroy() {
    if (dead) return;
    dead = true;
    for (const m of meshes) {
      gl.deleteBuffer(m.vbo);
      gl.deleteBuffer(m.ibo);
    }
    for (let k = 0; k < instBuf.length; k++) {
      if (instBuf[k]) gl.deleteBuffer(instBuf[k]);
      instBuf[k] = null;
      instCap[k] = 0;
    }
    if (hasVAO) {
      for (const s of slots) if (s && s.vao) deleteVAO(s.vao);
      deleteVAO(screenVAO);
    }
    gl.deleteProgram(program);
    gl.deleteProgram(sprogram);
  }

  return { draw, stats, destroy, instancing: isGL2 ? 'webgl2' : 'ANGLE_instanced_arrays', vertexArrays: hasVAO, screenPrecision: PF };
}
