#!/usr/bin/env python3
"""SCOPE-0: the explorer page, one self-contained HTML file, from the scope table.

  build_explorer.py TABLE_DIR OUT.html

The page draws only what the table holds. It computes nothing from the values but a colour.
"""
import json
import os
import sys

PAGE = r"""<title>Kindscope</title>
<meta name="claude:viewport-fit-sides" content="cover">
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500&family=IBM+Plex+Sans:wght@400;500;600&display=swap">
<style>
/* Layout: an instrument panel. Controls on top, the model drawn as a column of layers (depth runs down), the reading of one cell beside it. */
:root {
  --bg: #f6f6f3; --panel: #fcfcfb; --ink: #0b0b0b; --ink2: #52514e; --ink3: #7c7b76; --line: #dcdbd5; --focus: #1c5cab;
  --seq0: #e6effb; --seq1: #cde2fb; --seq2: #86b6ef; --seq3: #3987e5; --seq4: #1c5cab; --seq5: #123f78;
  --divn: #1c5cab; --divmid: #f0efec; --divp: #b8372f;
  --void: #e7e6e1; --hatch: rgba(11,11,11,.42); --empty: #cfcec8;
  --sans: "IBM Plex Sans", system-ui, -apple-system, "Segoe UI", sans-serif; --mono: "IBM Plex Mono", ui-monospace, Menlo, Consolas, monospace;
}
@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) {
  --bg: #121211; --panel: #1a1a19; --ink: #f0efec; --ink2: #c3c2b7; --ink3: #8f8e86; --line: #34342f; --focus: #86b6ef;
  --seq0: #1d2633; --seq1: #1b3350; --seq2: #184f95; --seq3: #2a78d6; --seq4: #86b6ef; --seq5: #cde2fb;
  --divn: #5a9df0; --divmid: #383835; --divp: #e2695f;
  --void: #2a2a27; --hatch: rgba(240,239,236,.5); --empty: #45453f; color-scheme: dark } }
:root[data-theme="dark"] {
  --bg: #121211; --panel: #1a1a19; --ink: #f0efec; --ink2: #c3c2b7; --ink3: #8f8e86; --line: #34342f; --focus: #86b6ef;
  --seq0: #1d2633; --seq1: #1b3350; --seq2: #184f95; --seq3: #2a78d6; --seq4: #86b6ef; --seq5: #cde2fb;
  --divn: #5a9df0; --divmid: #383835; --divp: #e2695f;
  --void: #2a2a27; --hatch: rgba(240,239,236,.5); --empty: #45453f; color-scheme: dark }
body { background: var(--bg); color: var(--ink); font-family: var(--sans); font-size: 14px; line-height: 1.45;
  padding-left: max(16px, env(safe-area-inset-left, 0px)); padding-right: max(16px, env(safe-area-inset-right, 0px)); padding-block: 20px 40px; }
.wrap { max-width: 1120px; margin: 0 auto; display: flex; flex-direction: column; gap: 16px; }
h1 { font-size: 22px; font-weight: 600; margin: 0; letter-spacing: -.01em; }
.lede { color: var(--ink2); max-width: 68ch; margin: 4px 0 0; }
.row { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
.lab { font-size: 11px; letter-spacing: .08em; text-transform: uppercase; color: var(--ink3); width: 100%; }
button.chip { font: inherit; color: var(--ink); background: var(--panel); border: 1px solid var(--line); border-radius: 4px; padding: 6px 10px; cursor: pointer; text-align: left; }
button.chip[aria-pressed="true"] { border-color: var(--ink); box-shadow: inset 0 0 0 1px var(--ink); }
button.chip:focus-visible, .cell:focus-visible { outline: 2px solid var(--focus); outline-offset: 2px; }
button.chip small { display: block; color: var(--ink3); font-size: 11px; }
.frame { border-left: 3px solid var(--ink3); padding: 6px 10px; background: var(--panel); color: var(--ink2); max-width: 90ch; }
.frame b { color: var(--ink); font-weight: 600; }
.main { display: grid; grid-template-columns: minmax(0, 1.25fr) minmax(0, 1fr); gap: 16px; align-items: start; }
@media (max-width: 760px) { .main { grid-template-columns: minmax(0, 1fr); } }
.panel { background: var(--panel); border: 1px solid var(--line); border-radius: 6px; padding: 14px; min-width: 0; }
.panel h2 { font-size: 15px; font-weight: 600; margin: 0 0 2px; text-wrap: balance; }
.kind { color: var(--ink2); margin: 0 0 10px; }
.gridwrap { overflow-x: auto; }
table.grid { border-collapse: separate; border-spacing: 2px; font-family: var(--mono); font-size: 11px; }
table.grid th { font-weight: 400; color: var(--ink3); padding: 0 6px 2px; text-align: center; white-space: nowrap; }
table.grid th.d { text-align: right; padding: 0 6px 0 0; }
.cell { width: 54px; min-width: 40px; border-radius: 2px; border: 0; padding: 0; display: block; cursor: pointer; position: relative; background: transparent; }
.cell.none { cursor: default; background: transparent; outline: 1px dashed var(--empty); outline-offset: -1px; }
.cell.exploratory::after { content: ""; position: absolute; inset: 0; border-radius: 2px; background: repeating-linear-gradient(45deg, transparent 0 4px, var(--hatch) 4px 5px); }
.cell.unconv::after { content: ""; position: absolute; inset: 0; border-radius: 2px; background: radial-gradient(var(--hatch) 1px, transparent 1.4px) 0 0 / 5px 5px; }
.cell.void { background: var(--void) !important; }
.cell.void::after { content: ""; position: absolute; inset: 0; background: linear-gradient(to top right, transparent calc(50% - .6px), var(--ink3) calc(50% - .6px) calc(50% + .6px), transparent calc(50% + .6px)); }
.cell.sel { box-shadow: 0 0 0 2px var(--panel), 0 0 0 4px var(--ink); z-index: 1; }
.legend { display: flex; flex-wrap: wrap; gap: 6px 16px; align-items: center; margin-top: 12px; color: var(--ink2); font-size: 12px; }
.ramp { width: 150px; height: 10px; border-radius: 2px; }
.sw { display: inline-flex; align-items: center; gap: 6px; }
.sw i { width: 18px; height: 12px; border-radius: 2px; display: inline-block; position: relative; }
.count { color: var(--ink3); font-size: 12px; margin-top: 8px; }
dl.read { display: grid; grid-template-columns: max-content minmax(0, 1fr); gap: 4px 12px; margin: 10px 0 0; }
dl.read dt { color: var(--ink3); } dl.read dd { margin: 0; overflow-wrap: anywhere; }
.big { font-family: var(--mono); font-size: 26px; font-weight: 500; font-variant-numeric: tabular-nums; }
.big small { font-size: 12px; color: var(--ink2); font-weight: 400; }
.pill { display: inline-block; border: 1px solid var(--line); border-radius: 3px; padding: 0 6px; font-size: 12px; }
.pill.preregistered { border-color: var(--ink); } .pill.void { text-decoration: line-through; color: var(--ink3); }
.mono { font-family: var(--mono); font-size: 12px; }
svg text { font-family: var(--mono); font-size: 10px; fill: var(--ink3); }
details { color: var(--ink2); } details summary { cursor: pointer; color: var(--ink); }
table.rows { border-collapse: collapse; font-size: 12px; font-variant-numeric: tabular-nums; }
table.rows th, table.rows td { text-align: left; padding: 2px 10px 2px 0; border-bottom: 1px solid var(--line); white-space: nowrap; }
.tip { position: fixed; pointer-events: none; background: var(--ink); color: var(--bg); padding: 4px 8px; border-radius: 3px; font-family: var(--mono); font-size: 11px; z-index: 20; max-width: 260px; }
.foot { color: var(--ink3); font-size: 12px; max-width: 90ch; }
</style>
<div class="wrap">
  <header>
    <h1>Kindscope</h1>
    <p class="lede">What has been measured inside two language models, site by site. Pick a model and one kind of reading. A site nobody measured is drawn empty: nothing is filled in between layers, carried from one model to the other, or averaged across kinds.</p>
  </header>
  <div class="row" id="models"><span class="lab">Model</span></div>
  <div class="frame" id="frame"></div>
  <div class="row" id="layers"><span class="lab">Reading</span></div>
  <div class="main">
    <section class="panel">
      <h2 id="ltitle"></h2>
      <p class="kind" id="lkind"></p>
      <div class="gridwrap"><table class="grid" id="grid"></table></div>
      <div class="legend" id="legend"></div>
      <div class="count" id="count"></div>
    </section>
    <section class="panel" id="detail"></section>
  </div>
  <section class="panel">
    <h2 id="ptitle">By depth</h2>
    <p class="kind" id="pkind"></p>
    <div class="gridwrap"><svg id="profile" width="720" height="210" viewBox="0 0 720 210" role="img"></svg></div>
    <details><summary>The same cells as a table</summary><div class="gridwrap"><table class="rows" id="rows"></table></div></details>
  </section>
  <p class="foot" id="foot"></p>
</div>
<div class="tip" id="tip" hidden></div>
<script>
const T = __TABLE__;
const SIDE = __SIDE__;
const $ = id => document.getElementById(id);
const css = n => getComputedStyle(document.documentElement).getPropertyValue(n).trim();
const hex = h => [1, 3, 5].map(i => parseInt(h.slice(i, i + 2), 16));
const mix = (a, b, t) => "rgb(" + a.map((x, i) => Math.round(x + (b[i] - x) * t)).join(",") + ")";
function colour(l, v) {
  const [a, b] = l.domain;
  if (a < 0) {
    const m = Math.max(Math.abs(a), Math.abs(b)), t = Math.max(-1, Math.min(1, v / m));
    return mix(hex(css("--divmid")), hex(css(t < 0 ? "--divn" : "--divp")), Math.abs(t));
  }
  const t = Math.max(0, Math.min(1, (v - a) / (b - a))) * 5, i = Math.min(4, Math.floor(t));
  return mix(hex(css("--seq" + i)), hex(css("--seq" + (i + 1))), t - i);
}
const fmt = v => Array.isArray(v) ? v.map(fmt).join(" to ") : (Math.abs(v) >= 100 ? v.toFixed(0) : Math.abs(v) >= 10 ? v.toFixed(1) : v.toFixed(3));
const siteName = s => "layer " + s.layer + " · " + s.component + (s.position ? " · " + s.position + " byte" : "");
const esc = s => String(s).replace(/[&<>"]/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
let state = { model: Object.keys(T.models)[0], layer: null, sel: null };
try { const h = location.hash.slice(1); if (T.layers.some(l => l.id.replace(/\./g, "-") === h)) { state.layer = h; } } catch (e) {}
const sub = r => r.site.position || r.category || null;
function layerOf(id) { return T.layers.find(l => l.id.replace(/\./g, "-") === id); }
function rowsOf(l) { return T.rows.filter(r => r.layer === l.id); }
function columns(l) {
  const m = T.models[l.model], rows = rowsOf(l), cols = [];
  for (const c of m.components) {
    const ps = [...new Set(rows.filter(r => r.site.component === c).map(sub))];
    for (const p of (ps.length ? ps : [null])) cols.push({ component: c, position: p });
  }
  return cols;
}
function render() {
  if (!state.layer || layerOf(state.layer).model !== state.model) state.layer = T.layers.find(l => l.model === state.model).id.replace(/\./g, "-");
  const l = layerOf(state.layer), m = T.models[l.model], rows = rowsOf(l), cols = columns(l);
  const at = (L, c) => rows.find(r => r.site.layer === L && r.site.component === c.component && sub(r) === c.position);
  if (!state.sel || !rows.includes(state.sel)) state.sel = rows.filter(r => r.standing !== "void").sort((a, b) => Math.abs(b.value) - Math.abs(a.value))[0] || rows[0];
  for (const b of $("models").querySelectorAll("button")) b.setAttribute("aria-pressed", b.dataset.id === state.model);
  $("frame").innerHTML = "<b>The instrument's frame is " + esc(m.frame) + " on this model.</b> " + esc(m.frame_reason) + ".";
  const lr = $("layers"); lr.querySelectorAll("button").forEach(b => b.remove());
  for (const x of T.layers.filter(x => x.model === state.model)) {
    const b = document.createElement("button"); b.className = "chip"; b.id = "layer-" + x.id.replace(/\./g, "-");
    b.setAttribute("aria-pressed", x.id === l.id); b.innerHTML = esc(x.title) + "<small>" + esc(x.question) + "</small>";
    b.onclick = () => { state.layer = x.id.replace(/\./g, "-"); state.sel = null; try { location.hash = state.layer; } catch (e) {} render(); };
    lr.appendChild(b);
  }
  $("ltitle").textContent = l.title; $("lkind").textContent = "Each cell: " + l.kind + ".";
  const h = m.layers > 12 ? 15 : 34, g = $("grid");
  let html = "<tr><th class='d'>depth</th>" + cols.map(c => "<th>" + esc(c.component) + (c.position ? "<br>" + esc(c.position) : "") + "</th>").join("") + "</tr>";
  for (let L = 0; L < m.layers; L++) {
    html += "<tr><th class='d'>" + L + "</th>";
    cols.forEach((c, ci) => {
      const r = at(L, c);
      if (!r) html += "<td><span class='cell none' style='height:" + h + "px' title='not measured'></span></td>";
      else html += "<td><button class='cell " + r.standing + (r.fit === "unconverged" ? " unconv" : "") + (r === state.sel ? " sel" : "") + "' style='height:" + h + "px;background:" + (r.standing === "void" ? "" : colour(l, r.value)) + "' data-i='" + T.rows.indexOf(r) + "' aria-label='" + esc(siteName(r.site) + ", " + (r.standing === "void" ? "void" : fmt(r.value) + " " + l.unit) + ", " + r.standing) + "'></button></td>";
    });
    html += "</tr>";
  }
  g.innerHTML = html;
  g.querySelectorAll("button.cell").forEach(b => {
    const r = T.rows[+b.dataset.i];
    b.onclick = () => { state.sel = r; render(); };
    b.onmousemove = e => { const t = $("tip"); t.hidden = false; t.textContent = siteName(r.site) + ": " + (r.standing === "void" ? "void" : fmt(r.value) + " " + l.unit) + " (" + r.standing + ")"; t.style.left = Math.min(e.clientX + 12, innerWidth - 270) + "px"; t.style.top = (e.clientY + 14) + "px"; };
    b.onmouseleave = () => { $("tip").hidden = true; };
  });
  const [a, b] = l.domain, stops = a < 0 ? [css("--divn"), css("--divmid"), css("--divp")] : [0, 1, 2, 3, 4, 5].map(i => css("--seq" + i));
  const has = s => rows.some(r => r.standing === s);
  $("legend").innerHTML = "<span class='sw'><span class='mono'>" + fmt(a < 0 ? -Math.max(Math.abs(a), Math.abs(b)) : a) + "</span><span class='ramp' style='background:linear-gradient(to right," + stops.join(",") + ")'></span><span class='mono'>" + fmt(a < 0 ? Math.max(Math.abs(a), Math.abs(b)) : b) + " " + esc(l.unit) + "</span></span>" +
    (has("preregistered") ? "<span class='sw'><i style='background:" + stops[stops.length - 2] + "'></i>solid: preregistered</span>" : "") +
    (has("exploratory") ? "<span class='sw'><i class='cell exploratory' style='width:18px;min-width:0;height:12px;display:inline-block;background:" + stops[stops.length - 2] + "'></i>hatched: exploratory</span>" : "") +
    (rows.some(r => r.fit === "unconverged") ? "<span class='sw'><i class='cell unconv' style='width:18px;min-width:0;height:12px;display:inline-block;background:" + stops[stops.length - 2] + "'></i>dotted: fit did not converge</span>" : "") +
    (has("void") ? "<span class='sw'><i class='cell void' style='width:18px;min-width:0;height:12px;display:inline-block'></i>struck: void, not a value</span>" : "") +
    "<span class='sw'><i style='outline:1px dashed var(--empty);outline-offset:-1px'></i>empty: not measured</span>";
  const total = m.layers * cols.length;
  $("count").textContent = rows.length + " of " + total + " sites in this view have a reading. Compared against: " + l.null + ".";
  detail(l, state.sel); profile(l, rows, state.sel);
  $("rows").innerHTML = "<tr><th>site</th><th>value</th><th>interval</th><th>compared against</th><th>standing</th><th>experiment</th></tr>" + rows.map(r => "<tr><td>" + esc(siteName(r.site) + (r.category ? " · " + r.category : "")) + "</td><td>" + (r.standing === "void" ? "<s>" + fmt(r.value) + "</s>" : fmt(r.value)) + "</td><td>" + (r.interval ? fmt(r.interval) : "") + "</td><td>" + (r.null ? fmt(r.null.value) : "") + "</td><td>" + r.standing + (r.fit === "unconverged" ? ", unconverged" : "") + (r.stake ? ", stake " + r.stake : "") + "</td><td>" + esc(r.source.experiment) + "</td></tr>").join("");
}
function detail(l, r) {
  if (!r) { $("detail").innerHTML = "<h2>Nothing measured in this view</h2>"; return; }
  const voided = r.standing === "void";
  $("detail").innerHTML = "<h2>" + esc(siteName(r.site)) + (r.category ? " · " + esc(r.category) : "") + "</h2>" +
    "<div class='big'>" + (voided ? "<s>" + fmt(r.value) + "</s>" : fmt(r.value)) + " <small>" + esc(l.unit) + "</small></div>" +
    "<dl class='read'>" +
    "<dt>standing</dt><dd><span class='pill " + r.standing + "'>" + r.standing + "</span> " + esc(T.standings[r.standing]) + "</dd>" +
    (r.fit && r.fit !== "n/a" ? "<dt>the fit</dt><dd><span class='pill'>" + r.fit + "</span> " + esc(T.fits[r.fit]) + "</dd>" : "") +
    (r.interval ? "<dt>95 % interval</dt><dd class='mono'>" + fmt(r.interval) + (r.n ? " (n = " + (Array.isArray(r.n) ? r.n.join(", ") : r.n) + ")" : "") + "</dd>" : "") +
    (r.null ? "<dt>compared against</dt><dd><span class='mono'>" + fmt(r.null.value) + "</span> " + esc(r.null.kind) + "</dd>" : "<dt>compared against</dt><dd>" + esc(r.bare || (r.interval ? "zero; see the interval" : "nothing stored beside this value")) + "</dd>") +
    (r.stake ? "<dt>the stake on it</dt><dd>" + esc(r.stake) + "</dd>" : "") +
    (r.note ? "<dt>note</dt><dd>" + esc(r.note) + "</dd>" : "") +
    "<dt>experiment</dt><dd>" + esc(r.source.experiment) + "</dd>" +
    "<dt>file and key</dt><dd class='mono'>" + esc(r.source.file) + " · " + esc(r.source.key) + "</dd>" +
    "<dt>from</dt><dd>" + esc(r.source.provenance) + "</dd></dl>";
}
function profile(l, rows, sel) {
  const svg = $("profile"), m = T.models[l.model];
  if (!sel) { svg.innerHTML = ""; return; }
  const col = rows.filter(r => r.site.component === sel.site.component && sub(r) === sub(sel)).sort((x, y) => x.site.layer - y.site.layer);
  $("ptitle").textContent = "By depth: " + sel.site.component + (sub(sel) ? ", " + sub(sel) : "");
  $("pkind").textContent = "The selected column down the model, with what each value is compared against (hollow marks). Gaps are sites with no reading.";
  const W = 720, H = 210, x0 = 46, x1 = W - 14, y0 = 14, y1 = H - 30;
  let lo = l.domain[0], hi = l.domain[1];
  if (lo < 0) { const mm = Math.max(Math.abs(lo), Math.abs(hi)); lo = -mm; hi = mm; }
  for (const r of col) { for (const v of [r.value].concat(r.interval || [], r.null ? [].concat(r.null.value) : [])) { lo = Math.min(lo, v); hi = Math.max(hi, v); } }
  const X = L => x0 + (x1 - x0) * (m.layers === 1 ? .5 : L / (m.layers - 1)), Y = v => y1 - (y1 - y0) * (v - lo) / (hi - lo);
  let s = "";
  for (const v of [lo, (lo + hi) / 2, hi]) s += "<line x1='" + x0 + "' x2='" + x1 + "' y1='" + Y(v) + "' y2='" + Y(v) + "' stroke='var(--line)' stroke-width='1'/><text x='" + (x0 - 6) + "' y='" + (Y(v) + 3) + "' text-anchor='end'>" + fmt(v) + "</text>";
  const step = m.layers > 12 ? 5 : 1;
  for (let L = 0; L < m.layers; L += step) s += "<text x='" + X(L) + "' y='" + (H - 12) + "' text-anchor='middle'>" + L + "</text>";
  s += "<text x='" + x1 + "' y='" + (H - 1) + "' text-anchor='end'>depth (layer)</text>";
  const live = col.filter(r => r.standing !== "void");
  let d = "", prev = null;
  for (const r of live) { d += (prev !== null && r.site.layer - prev === 1 ? "L" : "M") + X(r.site.layer) + " " + Y(r.value); prev = r.site.layer; }
  s += "<path d='" + d + "' fill='none' stroke='var(--ink2)' stroke-width='2'/>";
  for (const r of col) {
    const x = X(r.site.layer);
    if (r.null) for (const v of [].concat(r.null.value)) s += "<circle cx='" + x + "' cy='" + Y(v) + "' r='3' fill='var(--panel)' stroke='var(--ink3)' stroke-width='1.2'/>";
    if (r.interval) s += "<line x1='" + x + "' x2='" + x + "' y1='" + Y(r.interval[0]) + "' y2='" + Y(r.interval[1]) + "' stroke='var(--ink2)' stroke-width='1.2'/>";
    if (r.standing === "void") s += "<path d='M" + (x - 3) + " " + (Y(r.value) - 3) + "l6 6m0 -6l-6 6' stroke='var(--ink3)' stroke-width='1.2' fill='none'/>";
    else s += "<circle cx='" + x + "' cy='" + Y(r.value) + "' r='" + (r === sel ? 5.5 : 4) + "' fill='" + colour(l, r.value) + "' stroke='var(--ink)' stroke-width='" + (r === sel ? 2 : 1) + "'><title>" + esc(siteName(r.site) + ": " + fmt(r.value) + " " + l.unit) + "</title></circle>";
  }
  svg.innerHTML = s;
  svg.setAttribute("aria-label", l.title + " by depth for " + sel.site.component);
}
for (const [id, m] of Object.entries(T.models)) {
  const b = document.createElement("button"); b.className = "chip"; b.dataset.id = id; b.id = "model-" + id; b.textContent = m.title;
  b.onclick = () => { state.model = id; state.sel = null; render(); }; $("models").appendChild(b);
}
if (state.layer) state.model = layerOf(state.layer).model;
$("foot").textContent = "SCOPE-0, version " + SIDE.version + ": " + T.rows.length + " readings copied from result files of the RATCHET kindscope series (commit " + SIDE.ratchet + ") and of the earlier hand-over bundle. Nothing was measured or re-derived to draw this page. Which experiments feed each view was chosen by the CIRISHolon session and corrected once by the session that owns the instrument.";
render();
try { matchMedia("(prefers-color-scheme: dark)").addEventListener("change", render); new MutationObserver(render).observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] }); } catch (e) {}
</script>
"""


def main():
    d, out = sys.argv[1], sys.argv[2]
    t = json.load(open(os.path.join(d, "scope_table.json"), encoding="utf-8"))
    s = json.load(open(os.path.join(d, "sidecar.json"), encoding="utf-8"))
    for r in t["rows"]:
        r["value"] = float("%.6g" % r["value"])
        if r["interval"]:
            r["interval"] = [float("%.6g" % v) for v in r["interval"]]
    table = json.dumps(t, ensure_ascii=False, separators=(",", ":")).replace("</", "<\\/")
    side = json.dumps({"version": s["version"], "ratchet": s["provenance"]["ratchet_commit"][:12]})
    open(out, "w", encoding="utf-8").write(PAGE.replace("__TABLE__", table).replace("__SIDE__", side))
    print(out, os.path.getsize(out), "bytes")


if __name__ == "__main__":
    main()
