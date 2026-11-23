// The eye each backend derives from a vp (draw.js and draw-webgpu.js, function eyeFrom) against the preimage of clip
// (0, 0, -1, 0) found by a plain Gauss-Jordan inverse, over orthographic and perspective matrices with turns and unequal
// scales. Exit 1 when any differs by more than 1e-9 in direction (and in w). `--plant` checks the formula as it was before
// the CORRECTION of 2026-11-23 (the inverse's third row), which must fail.
//   node crates/holon-draw/tools/eye_check.mjs [--plant]
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
const WEB = join(dirname(fileURLToPath(import.meta.url)), '..', 'web');
const plant = process.argv.includes('--plant');
function extract(file) {
  let src = readFileSync(join(WEB, file), 'utf8');
  const i = src.indexOf('  function eyeFrom(m) {'), j = src.indexOf('\n  }\n', i) + 4;
  let f = src.slice(i, j);
  if (plant) f = f.replace(/let x = .*\n.*let y = .*\n.*let z = .*\n.*let w = .*;/, `let x = -(a31 * b05 - a32 * b04 + a33 * b03) / det;
    let y = -(a32 * b02 - a30 * b05 - a33 * b01) / det;
    let z = -(a30 * b04 - a31 * b02 + a33 * b00) / det;
    let w = -(a31 * b01 - a30 * b03 - a32 * b00) / det;`);
  return new Function('inv', f + '\nreturn eyeFrom;');
}
function preimage(m) {
  const A = [];
  for (let r = 0; r < 4; r++) { A.push([]); for (let c = 0; c < 4; c++) A[r].push(m[c * 4 + r]); for (let c = 0; c < 4; c++) A[r].push(r === c ? 1 : 0); }
  for (let i = 0; i < 4; i++) {
    let p = i; for (let r = i; r < 4; r++) if (Math.abs(A[r][i]) > Math.abs(A[p][i])) p = r;
    [A[i], A[p]] = [A[p], A[i]]; const d = A[i][i]; for (let c = 0; c < 8; c++) A[i][c] /= d;
    for (let r = 0; r < 4; r++) if (r !== i) { const f = A[r][i]; for (let c = 0; c < 8; c++) A[r][c] -= f * A[i][c]; }
  }
  return [0, 1, 2, 3].map((r) => -A[r][6]);
}
const mul = (a, b) => { const o = new Array(16).fill(0); for (let c = 0; c < 4; c++) for (let r = 0; r < 4; r++) for (let k = 0; k < 4; k++) o[c * 4 + r] += a[k * 4 + r] * b[c * 4 + k]; return o; };
const rot = (yaw, pitch, roll) => { const cy = Math.cos(yaw), sy = Math.sin(yaw), cp = Math.cos(pitch), sp = Math.sin(pitch), cr = Math.cos(roll), sr = Math.sin(roll);
  const Y = [cy, 0, -sy, 0, 0, 1, 0, 0, sy, 0, cy, 0, 0, 0, 0, 1], X = [1, 0, 0, 0, 0, cp, sp, 0, 0, -sp, cp, 0, 0, 0, 0, 1], Z = [cr, sr, 0, 0, -sr, cr, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1];
  return mul(Z, mul(X, Y)); };
let seed = 7; const rnd = () => ((seed = (seed * 1103515245 + 12345) % 2147483648) / 2147483648);
const cases = [];
for (let k = 0; k < 200; k++) {
  const R = rot(6 * rnd(), 3 * rnd() - 1.5, 6 * rnd()), t = [10 * rnd() - 5, 10 * rnd() - 5, -20 * rnd() - 2];
  const V = R.slice(); V[12] = t[0]; V[13] = t[1]; V[14] = t[2];
  const ax = 0.2 + 2 * rnd(), ay = 0.2 + 2 * rnd(), az = (rnd() < 0.5 ? -1 : 1) * (0.05 + rnd());
  const O = [ax, 0, 0, 0, 0, ay, 0, 0, 0, 0, az, 0, 0, 0, 0, 1];
  const f = 0.5 + rnd(), asp = 0.5 + rnd(), n = 0.1, fa = 100;
  const P = [f / asp, 0, 0, 0, 0, f, 0, 0, 0, 0, (fa + n) / (n - fa), -1, 0, 0, 2 * fa * n / (n - fa), 0];
  cases.push(['ortho', mul(O, V)], ['perspective', mul(P, V)]);
}
let bad = 0;
for (const file of ['draw.js', 'draw-webgpu.js']) {
  const inv = new Float64Array(16), eyeFrom = extract(file)(inv);
  let worst = 0, n = 0;
  for (const [kind, m] of cases) {
    eyeFrom(m);
    const e = preimage(m), got = [inv[0], inv[1], inv[2], inv[3]];
    let d;
    if (Math.abs(e[3]) > 1e-12) d = Math.hypot(got[0] / got[3] - e[0] / e[3], got[1] / got[3] - e[1] / e[3], got[2] / got[3] - e[2] / e[3]) / Math.max(1, Math.hypot(e[0] / e[3], e[1] / e[3], e[2] / e[3]));
    else { const l = Math.hypot(e[0], e[1], e[2]); d = Math.hypot(got[0] - e[0] / l, got[1] - e[1] / l, got[2] - e[2] / l) + Math.abs(got[3]); }
    worst = Math.max(worst, d); n++;
    if (!(d <= 1e-9)) bad++;
  }
  console.log(`${file}: ${n} matrices (${n / 2} orthographic, ${n / 2} perspective), largest difference from the preimage ${worst.toExponential(2)}`);
}
console.log(plant ? '(planted: the formula before the CORRECTION)' : '', bad ? `FAIL: ${bad} matrices` : 'PASS');
process.exit(bad ? 1 : 0);
