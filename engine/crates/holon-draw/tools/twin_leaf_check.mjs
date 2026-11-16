// Is `holon_draw::twin::build_leaf` the same vertex stream as the twin's own `tlBuildLeaf`?
//
//   node twin_leaf_check.mjs <twin's docs/tiling.js> <the twin_leaf example binary, or holon_draw.wasm> [leaves]
//
// Given the .wasm it goes through the browser boundary and also times both builders. With
// PLANT=1 the wasm side is given a wrong household size, and the check must fail.
//
// The twin's function is READ from the twin's tree at run time and run as it is, with its
// kernel replaced by synthetic layout buffers made here (so this checks the builder, not the
// kernel). Nothing of the twin's is copied into this repository and nothing is written.
// Prints how many leaves, vertices and floats were compared, and how many floats differ and
// by how much. Exit 0 only if every leaf has the same number of floats and none differs by
// more than 1 unit in the last place of a 32-bit float.
import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";

const [, , tilingPath, binary, nArg] = process.argv;
const N = Number(nArg || 400);
const src = readFileSync(tilingPath, "utf8");
const grab = (re, what) => { const m = src.match(re); if (!m) { console.error("not found in the twin's file: " + what); process.exit(2); } return m[0]; };
const fn = (() => { // the function's text, by brace matching from its header
  const at = src.indexOf("function tlBuildLeaf(leaf){");
  if (at < 0) { console.error("tlBuildLeaf not found"); process.exit(2); }
  let depth = 0, i = src.indexOf("{", at);
  for (; i < src.length; i++) { if (src[i] === "{") depth++; else if (src[i] === "}") { depth--; if (depth === 0) break; } }
  return src.slice(at, i + 1);
})();
// the twin's one style array (97 numbers in holon-draw's STYLE_FIELDS order), when its
// tlBuildLeaf reads its constants from it; optional, so the check runs on either form
const styleLine = (src.match(/const TL_STYLE = [^\n]*\n/) || [""])[0];
const consts = [
  grab(/const tlTone = [^\n]*\n/, "tlTone"),
  grab(/const TL_ROAD = [^\n]*\n/, "TL_ROAD"),
  grab(/const TL_ROOF_TONES = [^\n]*\n/, "TL_ROOF_TONES"),
  styleLine,
].join("");
const TL_STYLE = styleLine ? new Function(styleLine + "return TL_STYLE;")() : null;
if (TL_STYLE && TL_STYLE.length !== 97) { console.error("TL_STYLE has " + TL_STYLE.length + " numbers, not 97"); process.exit(2); }

// a small deterministic generator (not the twin's hash)
let s = 0x2545f491;
const rnd = () => { s ^= s << 13; s >>>= 0; s ^= s >>> 17; s ^= s << 5; s >>>= 0; return s / 4294967296; };
const pick = (a) => a[Math.floor(rnd() * a.length)];

function makeLeaf(k) {
  const w = 40 + rnd() * 400, h = 40 + rnd() * 400;
  const hh = pick([0, 1, 3, 12, 60, 200, 900, 4000]) * (0.5 + rnd());
  const nR = Math.floor(rnd() * 9), nB = Math.floor(rnd() * 40);
  const R = [], S = [], B = [], TN = [];
  for (let i = 0; i < nR; i++) {
    const x0 = (rnd() - 0.5) * w, y0 = (rnd() - 0.5) * h;
    const zero = rnd() < 0.05;
    R.push(x0, y0, zero ? x0 : (rnd() - 0.5) * w, zero ? y0 : (rnd() - 0.5) * h, pick([4, 5, 6.5, 7, 9, 11, 14]));
    S.push(pick([0, 1]), pick([0, 1, 2, 3, 4]), pick([0, 1, 2, 3, 4]));
  }
  for (let i = 0; i < nB; i++) {
    B.push((rnd() - 0.5) * w, (rnd() - 0.5) * h, 3 + rnd() * 30, 3 + rnd() * 30, rnd() * Math.PI * 2 - Math.PI, Math.floor(rnd() * 12), pick([0, 0, 1, 2, 8, 40]), Math.floor(rnd() * Math.max(1, nR)));
    TN.push(pick([0, 0, 0, 1, 2, 3, 4, 5, 6, 7]));
  }
  const tones = k % 3 === 0 ? null : TN;
  const cls = k % 4;
  const level = 3 + (k % 6);
  const id = level * 2 ** 47 + (Math.floor(rnd() * 2 ** 22) + 2 ** 22) * 2 ** 24 + Math.floor(rnd() * 2 ** 24);
  const laT = (rnd() - 0.5) * 150, loT = (rnd() - 0.5) * 340;
  const laL = laT + (rnd() - 0.5) * 0.02, loL = loT + (rnd() - 0.5) * 0.02;
  // S sometimes shorter than the roads, as the twin's code allows for
  const Sx = k % 11 === 0 ? S.slice(0, Math.max(0, S.length - 3)) : S;
  return { id, R, B, meta: [w, h, hh, cls], S: Sx, tones, cls, laT, loT, laL, loL };
}

const leaves = Array.from({ length: N }, (_, k) => makeLeaf(k));

// run the twin's builder over each leaf
const mem = { buffer: new ArrayBuffer(1 << 20) };
let cur = null;
const layout = () => { // place the buffers in `mem` and remember where
  let at = 1024; const where = {};
  for (const [i, a] of [[1, cur.R], [2, cur.B], [4, cur.meta], [6, cur.S], [8, cur.tones || []]]) { new Float64Array(mem.buffer, at, a.length).set(a); where[i] = [at, a.length]; at += 8 * a.length + 8; }
  cur.where = where;
};
const wasm = { holon_ctx_ptr: () => 0, holon_layout: () => layout(), holon_buf_ptr: (i) => (cur.where[i] || [0, 0])[0], holon_buf_len: (i) => (cur.where[i] || [0, 0])[1] };
const HOLON = { pack: (x) => x.id, ancestor: (leaf) => ({ id: -1, tile: true, of: leaf }) };
const tlCentre = (x) => x.tile ? [x.of.laT, x.of.loT] : [x.laL, x.loL];
const tlTree = () => null, tlContext = () => new Float64Array(33);
const TL_PPH = 2.5;
const tlBuildLeaf = new Function("wasm", "mem", "HOLON", "tlCentre", "tlTree", "tlContext", "TL_PPH", consts + fn + "\nreturn tlBuildLeaf;")(wasm, mem, HOLON, tlCentre, tlTree, tlContext, TL_PPH);
const TL_ROAD = new Function(consts + "return TL_ROAD;")();

const theirs = [], lines = [];
for (const L of leaves) {
  cur = L;
  theirs.push(tlBuildLeaf(L).data);
  const cT = Math.cos(L.laT * Math.PI / 180), sx = cT / Math.cos(L.laL * Math.PI / 180), ox = (L.loL - L.loT) * 111320 * cT, oy = (L.laL - L.laT) * 111320;
  const g = (a) => a.length ? Array.from(a).map((v) => v.toPrecision(17)).join(",") : "-";
  lines.push([L.id.toPrecision(17), g([ox, oy, sx]), g(TL_ROAD[L.cls]), g(L.R), g(L.B), g(L.meta), g(L.S), L.tones ? g(L.tones) : "-", String(TL_PPH), TL_STYLE ? g(TL_STYLE) : "-"].join(" "));
}
let ours, timing = null;
if (binary.endsWith(".wasm")) {
  // the browser artifact itself, through its boundary (src/abi.rs), and the time each side takes
  const { instance } = await WebAssembly.instantiate(readFileSync(binary), {});
  const x = instance.exports;
  if (TL_STYLE) { // hand the wasm the same numbers the twin's builder reads
    const p = x.hd_in(5, TL_STYLE.length); new Float64Array(x.memory.buffer, p, TL_STYLE.length).set(TL_STYLE);
    if (x.hd_style() !== 1) { console.error("the wasm refused TL_STYLE"); process.exit(2); }
  }
  const put = (which, a) => { const p = x.hd_in(which, a.length); new Float64Array(x.memory.buffer, p, a.length).set(a); };
  const build = (L, keep) => {
    const cT = Math.cos(L.laT * Math.PI / 180), sx = cT / Math.cos(L.laL * Math.PI / 180), ox = (L.loL - L.loT) * 111320 * cT, oy = (L.laL - L.laT) * 111320;
    put(0, L.R); put(1, L.B); put(2, L.meta); put(3, L.S); put(4, L.tones || []);
    const c = TL_ROAD[L.cls] || TL_ROAD[0], pph = process.env.PLANT ? 2.6 : TL_PPH;
    const n = x.hd_build_leaf(L.id, ox, oy, sx, pph, c[0], c[1], c[2], L.tones ? 1 : 0);
    const v = new Float32Array(x.memory.buffer, x.hd_out(), n);
    return keep ? v.slice() : v; // the twin keeps a copy too (its Float32Array(V))
  };
  const hexOf = (v) => { const d = new DataView(v.buffer, v.byteOffset, v.byteLength); let h = ""; for (let i = 0; i < v.length; i++) h += d.getUint32(4 * i, true).toString(16).padStart(8, "0"); return h; };
  ours = leaves.map((L) => hexOf(build(L, true)));
  const med = (f) => { const t = []; for (let r = 0; r < 7; r++) { const t0 = performance.now(); f(); t.push(performance.now() - t0); } t.sort((a, b) => a - b); return t[3]; };
  const js = med(() => { for (const L of leaves) { cur = L; tlBuildLeaf(L); } });
  const ws = med(() => { for (const L of leaves) build(L, true); });
  timing = { twin_js_ms_per_leaf: +(js / leaves.length).toFixed(4), wasm_ms_per_leaf: +(ws / leaves.length).toFixed(4), ratio: +(js / ws).toFixed(2), note: "node on this machine, unthrottled, median of 7 passes; not the twin's protocol (a phone-sized page at 6x throttle)" };
} else {
  const run = spawnSync(binary, [], { input: lines.join("\n") + "\n", maxBuffer: 1 << 30, encoding: "utf8" });
  if (run.status !== 0) { console.error("the builder failed: " + run.stderr); process.exit(2); }
  ours = run.stdout.trim().split("\n");
}

const bits = new DataView(new ArrayBuffer(4));
let floats = 0, verts = 0, differ = 0, worst = 0, lengthMismatch = 0, leavesDiffering = 0;
for (let k = 0; k < leaves.length; k++) {
  const a = theirs[k], hex = ours[k] || "";
  if (hex.length !== a.length * 8) { lengthMismatch++; continue; }
  let any = false;
  for (let i = 0; i < a.length; i++) {
    bits.setFloat32(0, a[i]); const x = bits.getUint32(0), y = parseInt(hex.slice(8 * i, 8 * i + 8), 16);
    if (x !== y) { differ++; any = true; worst = Math.max(worst, Math.abs(x - y)); }
  }
  if (any) leavesDiffering++;
  floats += a.length; verts += a.length / 10;
}
console.log(JSON.stringify({ style: TL_STYLE ? "the twin's TL_STYLE, given to both builders" : "the twin's inline constants against holon-draw's defaults", leaves: leaves.length, vertices: verts, floats, floats_differing: differ, leaves_differing: leavesDiffering, worst_difference_in_last_place: worst, leaves_of_another_length: lengthMismatch, timing }));
process.exit(lengthMismatch === 0 && worst <= 1 ? 0 : 1);
