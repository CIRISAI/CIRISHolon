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
  --void: #e7e6e1; --hatch: rgba(11,11,11,.42); --empty: #cfcec8; --s1: #2a78d6; --s2: #eb6834; --s3: #1baf7a; --s4: #eda100; --s7: #6250d6;
  --sans: "IBM Plex Sans", system-ui, -apple-system, "Segoe UI", sans-serif; --mono: "IBM Plex Mono", ui-monospace, Menlo, Consolas, monospace;
}
@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) {
  --bg: #121211; --panel: #1a1a19; --ink: #f0efec; --ink2: #c3c2b7; --ink3: #8f8e86; --line: #34342f; --focus: #86b6ef;
  --seq0: #1d2633; --seq1: #1b3350; --seq2: #184f95; --seq3: #2a78d6; --seq4: #86b6ef; --seq5: #cde2fb;
  --divn: #5a9df0; --divmid: #383835; --divp: #e2695f;
  --void: #2a2a27; --hatch: rgba(240,239,236,.5); --empty: #45453f; --s1: #3987e5; --s2: #d95926; --s3: #199e70; --s4: #c98500; --s7: #9085e9; color-scheme: dark } }
:root[data-theme="dark"] {
  --bg: #121211; --panel: #1a1a19; --ink: #f0efec; --ink2: #c3c2b7; --ink3: #8f8e86; --line: #34342f; --focus: #86b6ef;
  --seq0: #1d2633; --seq1: #1b3350; --seq2: #184f95; --seq3: #2a78d6; --seq4: #86b6ef; --seq5: #cde2fb;
  --divn: #5a9df0; --divmid: #383835; --divp: #e2695f;
  --void: #2a2a27; --hatch: rgba(240,239,236,.5); --empty: #45453f; --s1: #3987e5; --s2: #d95926; --s3: #199e70; --s4: #c98500; --s7: #9085e9; color-scheme: dark }
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
#scene { width: 100%; height: 540px; display: block; touch-action: none; cursor: grab; border-radius: 4px; background: var(--bg); }
#scene:active { cursor: grabbing; }
.found { display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); gap: 8px; margin-top: 12px; }
.found button { font: inherit; color: var(--ink); background: var(--panel); border: 1px solid var(--line); border-radius: 4px; padding: 10px 12px; text-align: left; cursor: pointer; display: flex; flex-direction: column; gap: 4px; }
.found button[aria-pressed="true"] { border-color: var(--ink); box-shadow: inset 0 0 0 1px var(--ink); }
.found button:focus-visible { outline: 2px solid var(--focus); outline-offset: 2px; }
.found b { font-weight: 600; } .found span { color: var(--ink2); } .found em { font-style: normal; font-size: 11px; letter-spacing: .06em; text-transform: uppercase; color: var(--ink3); }
.ctl { display: flex; flex-wrap: wrap; gap: 8px 16px; align-items: center; margin: 8px 0; color: var(--ink2); }
.ctl input[type=range] { width: 220px; max-width: 100%; }
.say { font-size: 16px; max-width: 70ch; margin: 10px 0 0; min-height: 3em; }
details.more > summary { font-weight: 600; font-size: 15px; padding: 6px 0; }
details.more > .inner { display: flex; flex-direction: column; gap: 16px; margin-top: 10px; }
.bytes { display: flex; flex-wrap: wrap; gap: 1px; font-family: var(--mono); font-size: 13px; line-height: 1; margin: 8px 0; }
.bytes button { font: inherit; color: var(--ink); border: 0; border-radius: 2px; padding: 4px 0; width: 1.05em; text-align: center; cursor: pointer; }
.bytes button.cur { box-shadow: 0 0 0 2px var(--ink); position: relative; z-index: 1; }
.bytes button:focus-visible { outline: 2px solid var(--focus); outline-offset: 1px; }
.two { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); gap: 16px; }
@media (max-width: 760px) { .two { grid-template-columns: minmax(0, 1fr); } }
.key { display: flex; flex-wrap: wrap; gap: 4px 14px; font-size: 12px; color: var(--ink2); }
.key i { display: inline-block; width: 14px; height: 3px; vertical-align: middle; margin-right: 5px; border-radius: 2px; }
.ok { border-left: 3px solid var(--ink); padding: 4px 10px; color: var(--ink2); margin-top: 10px; }
</style>
<div class="wrap">
  <header>
    <h1>Kindscope</h1>
    <p class="lede">A kind is a kind of <b>change</b>, so this starts from a change. Take two real story openings that differ in one word. Feed both to a small language model and ask, at each of its 13 stages and at each character after the change: how differently is the model now holding the two? Zero is identical. About one is as different as two unrelated texts. Press play to watch the difference spread.</p>
  </header>
  <section class="panel">
    <div class="ctl"><span class="lab" style="width:auto">What changed</span><span id="pv-sorts" class="row"></span></div>
    <div class="ctl"><button class="chip" id="pv-prev">Previous pair</button><button class="chip" id="pv-next">Next pair</button><span id="pv-count" style="color:var(--ink3)"></span></div>
    <p id="pv-text" style="font-family:var(--mono);font-size:14px;line-height:1.7;margin:6px 0"></p>
    <canvas id="wake" style="width:100%;height:500px;display:block;touch-action:none;cursor:grab;border-radius:4px;background:var(--bg)" aria-label="how differently the model holds the two versions, by stage (front to back) and by characters after the change (left to right)"></canvas>
    <div class="ctl">
      <button class="chip" id="pv-play" aria-pressed="false">Play through the stages</button>
      <label>Stage <input type="range" id="pv-stage" min="0" max="12" step="0.01" value="12"></label>
      <span class="sw"><span class="mono">0 same</span><span class="ramp" id="pv-ramp"></span><span class="mono">1 unrelated</span></span>
    </div>
    <p class="say" id="pv-say"></p>
    <p class="kind" id="pv-honest" style="margin:6px 0 0"></p>
  </section>
  <details class="more"><summary>What the model holds at each character (a view of characters, not of change)</summary><div class="inner">
  <p class="lede" style="margin:0">Every dot is one character of a story, placed by what the model is holding at that character; the cloud changes shape through the 13 stages: stripes, then a ring, then clusters. This view sorts characters, which is not what a kind is; it is kept for what it shows about the model’s position signal.</p>
  <section class="panel">
    <div class="lab" style="margin-bottom:4px">The filmstrip: the same cloud after each of the 13 stages. Click a frame.</div>
    <div id="c-film" style="display:flex;flex-wrap:wrap;gap:6px;margin-bottom:10px"></div>
    <canvas id="cloud" style="width:100%;height:560px;display:block;touch-action:none;cursor:grab;border-radius:4px;background:var(--bg)" aria-label="a cloud of points, one per character, changing shape through the model's 13 stages"></canvas>
    <div class="ctl">
      <button class="chip" id="c-play" aria-pressed="false">Play through the stages</button>
      <label>Stage <input type="range" id="c-stage" min="0" max="12" step="0.01" value="0"></label>
      <span class="row"><button class="chip" id="c-place" aria-pressed="true">Colour: place in the word</button><button class="chip" id="c-pos" aria-pressed="false">Colour: position in the story</button><button class="chip" id="c-surprise" aria-pressed="false">Colour: how hard the next character was</button></span>
    </div>
    <div class="key" id="c-key"></div>
    <p class="say" id="c-say"></p>
    <p class="kind" id="c-honest" style="margin:6px 0 0"></p>
  </section>
  </div></details>
  <details class="more"><summary>The same model as blocks: what each stage adds, what breaks without it, what it reads from</summary><div class="inner">
  <p class="lede" style="margin:0">Letters run left to right, the 13 stages front to back. Click a block to see which earlier characters each looking-back stage reads from there. Switch what the height shows: how much each stage adds, or how much breaks if you take that stage out. Pick a finding to see where in the model it lives.</p>
  <section class="panel">
    <canvas id="scene" aria-label="the model reading a story, in three dimensions: letters left to right, stages front to back, height is how much each stage added"></canvas>
    <div class="ctl">
      <span id="s3items" class="row"></span>
      <label>Move along the story <input type="range" id="s3win" min="0" max="208" value="13"></label>
      <span class="row"><button class="chip" id="s3h-adds" aria-pressed="true">Height: how much it adds</button><button class="chip" id="s3h-cost" aria-pressed="false">Height: what breaks without it</button></span>
      <button class="chip" id="s3spin" aria-pressed="false">Turn slowly</button>
      <span><i style="display:inline-block;width:12px;height:12px;border-radius:2px;background:var(--s1);vertical-align:-2px"></i> a stage that looks back at earlier letters &nbsp; <i style="display:inline-block;width:12px;height:12px;border-radius:2px;background:var(--s2);vertical-align:-2px"></i> a stage that rewrites what it has</span>
    </div>
    <p class="say" id="s3say"></p>
    <div class="found" id="s3found"></div>
  </section>
  </div></details>
  <details class="more"><summary>The measurements behind this, cell by cell</summary><div class="inner">
  <p class="lede" style="margin:0">Pick a model and one kind of reading. A site nobody measured is drawn empty: nothing is filled in between layers, carried from one model to the other, or averaged across kinds.</p>
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
  <section class="panel" id="pass">
    <h2>One prompt, passing through GoldWorm</h2>
    <p class="kind">Stand at one byte of a story and look down the model. Each block adds something to a running state; the lengths below are what each added here. The shading under the text is how hard the next byte was to predict.</p>
    <div class="row" id="items"><span class="lab">Story</span></div>
    <div class="bytes" id="bytes" aria-label="the story, one cell a byte"></div>
    <div class="two">
      <div><h2 id="wtitle"></h2><p class="kind">How much each block wrote at this byte, in the order the model adds them (depth runs down). Length is how much a block wrote here, not how much it matters: the last block writes most, and removing the first rewriting block does eight times the damage.</p><svg id="wbars" width="420" height="300" viewBox="0 0 420 300" role="img"></svg></div>
      <div><h2>Reading the same site four ways</h2><p class="kind">These readings mean something only at one kind of place: the byte just before a word that follows “the” or “a” (outlined in the text above). There, after each rewriting stage, how far the state sits from a typical such place, in standard deviations. Above zero leans toward an adjective coming next.</p>
        <div class="key"><span><i style="background:var(--s1)"></i>where an outside reader looks</span><span><i style="background:var(--s2)"></i>where the model writes</span><span><i style="background:var(--s3)"></i>the written part that is not a guess at the next letter</span><span><i style="background:var(--ink3)"></i>the guess at the letter that did come next</span></div>
        <svg id="rlines" width="420" height="250" viewBox="0 0 420 250" role="img"></svg></div>
    </div>
    <div class="ok" id="okline"></div>
  </section>
  </div></details>
  <p class="foot" id="foot"></p>
</div>
<div class="tip" id="tip" hidden></div>
<script>
const T = __TABLE__;
const CL = __SHAPES__;
const PR = __PAIRS__;
const TR = __TRACES__;
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
// ---------------------------------------------------------------- a change travelling through the model
const PV = { sort: null, idx: 0, k: 12, yaw: -0.55, pitch: 0.5, play: false };
const PSORTS = [...new Set(PR.pairs.map(p => p.sort))];
const SORT_SAY = { "name": "A name", "quality word": "A describing word", "thing": "The thing itself", "action word": "The wording only", "punctuation": "A comma or full stop", "number": "A number", "nothing changed": "Nothing (the check)" };
function pvList() { return PR.pairs.filter(p => p.sort === PV.sort); }
function wake() {
  const list = pvList(), p = list[PV.idx % list.length], cv = $("wake"), dpr = Math.min(2, window.devicePixelRatio || 1), Wp = cv.clientWidth || 900, Hp = cv.clientHeight || 500;
  if (cv.width !== Math.round(Wp * dpr)) { cv.width = Math.round(Wp * dpr); cv.height = Math.round(Hp * dpr); }
  const g = cv.getContext("2d"); g.setTransform(dpr, 0, 0, dpr, 0, 0); g.clearRect(0, 0, Wp, Hp);
  $("pv-count").textContent = "pair " + (PV.idx % list.length + 1) + " of " + list.length;
  const mark = (t, sp) => esc(t.slice(15, sp[0])) + "<b style='background:var(--seq1);padding:1px 3px;border-radius:2px'>" + esc(t.slice(sp[0], sp[1])) + "</b>" + esc(t.slice(sp[1]));
  if (p.null) {
    $("pv-text").innerHTML = "A: " + esc(p.text[0].slice(15)) + "<br>B: " + esc(p.text[1].slice(15));
    g.font = "14px " + css("--sans"); g.fillStyle = css("--ink2"); g.textAlign = "center";
    g.fillText("Nothing changed, so the model holds the two identically: the difference is " + p.largest_difference + " at each of " + p.positions + " characters and 13 stages.", Wp / 2, Hp / 2);
    $("pv-say").textContent = "This pair is the check on the picture. The two openings are the same sentence, so any difference shown would be a fault in the drawing or the instrument. There is none.";
    $("pv-honest").textContent = ""; return;
  }
  $("pv-text").innerHTML = "A: " + mark(p.text[0], p.changed_span[0]) + "<br>B: " + mark(p.text[1], p.changed_span[1]);
  const K = PR.stages.length, J = p.state[0].length, sc = Math.min(Wp / (Math.max(J, 6) * 2.3), Hp / 30), dx = 1.7, dz = 1.5, HH = 9;
  const cy = Math.cos(PV.yaw), sy = Math.sin(PV.yaw), cp = Math.cos(PV.pitch), sp = Math.sin(PV.pitch);
  const P = (x, y, z) => { const xr = x * cy - z * sy, zr = x * sy + z * cy; return [Wp / 2 + sc * xr, Hp * 0.66 - sc * (y * cp - zr * sp), zr * cp + y * sp]; };
  const seqc = [0, 1, 2, 3, 4, 5].map(i => hex(css("--seq" + i))), colour = v => { const u = Math.max(0, Math.min(1, v)) * 5, i = Math.min(4, Math.floor(u)); return mix(seqc[i], seqc[i + 1], u - i); };
  const upto = PV.k, val = (k, j) => { const f = Math.max(0, Math.min(1, upto - k + 1)); return k > Math.ceil(upto) ? 0 : p.state[k][j] * f; };
  const X = j => (j - (J - 1) / 2) * dx, Z = k => (k - (K - 1) / 2) * dz, quads = [];
  for (let k = 0; k < K; k++) for (let j = 0; j < J; j++) {
    const v = val(k, j), h = v * HH, x0 = X(j) - dx * .42, x1 = X(j) + dx * .42, z0 = Z(k) - dz * .42, z1 = Z(k) + dz * .42;
    quads.push({ d: P(X(j), h / 2, Z(k))[2], v, pts: [[x0, z0], [x1, z0], [x1, z1], [x0, z1]], h });
  }
  quads.sort((a, b) => b.d - a.d);
  const bg = hex(css("--bg"));
  for (const q of quads) {
    const top = q.pts.map(([x, z]) => P(x, q.h, z)), bot = q.pts.map(([x, z]) => P(x, 0, z)), c = colour(q.v).match(/\d+/g).map(Number);
    const face = (a, b, t) => { g.beginPath(); g.moveTo(top[a][0], top[a][1]); g.lineTo(top[b][0], top[b][1]); g.lineTo(bot[b][0], bot[b][1]); g.lineTo(bot[a][0], bot[a][1]); g.closePath(); g.fillStyle = mix(c, bg, t); g.fill(); };
    if (q.h > 0.02) { if (sy < 0) face(1, 2, .4); else face(3, 0, .4); if (cy > 0) face(0, 1, .25); else face(2, 3, .25); }
    g.beginPath(); top.forEach((t, i) => i ? g.lineTo(t[0], t[1]) : g.moveTo(t[0], t[1])); g.closePath(); g.fillStyle = q.h > 0.02 ? mix(c, bg, 0) : css("--line"); g.fill();
  }
  g.font = "12px " + css("--mono"); g.textBaseline = "middle"; g.textAlign = "center"; g.fillStyle = css("--ink2");
  const fz = cy > 0 ? Z(0) - dz * 1.1 : Z(K - 1) + dz * 1.1;
  for (let j = 0; j < J; j++) { const ch = j === 0 ? "■" : (p.after[j - 1] === " " ? "·" : p.after[j - 1]), q = P(X(j), 0, fz); g.fillText(ch, q[0], q[1]); }
  const sx = sy < 0 ? X(0) - dx * 1.2 : X(J - 1) + dx * 1.2; g.textAlign = sy < 0 ? "right" : "left"; g.fillStyle = css("--ink3");
  for (let k = 0; k < K; k++) { const q = P(sx, 0, Z(k)); g.fillText(String(k + 1), q[0], q[1]); }
  const kn = Math.round(PV.k), at = p.state[kn][0], nx = J > 1 ? p.state[kn][1] : null, n1 = J > 1 ? p.state[1][1] : null, far = J > 4 ? p.state[K - 1][J - 1] : null;
  let say = "■ is the last letter of the changed word (" + esc(p.changed_word[0]) + " / " + esc(p.changed_word[1]) + "); to its right, the characters that follow. Stage " + (kn + 1) + " of 13: at the changed word the model holds the two versions " + at.toFixed(2) + " apart";
  if (nx !== null) say += ", and one character later " + nx.toFixed(2) + ". ";
  else say += ". ";
  if (p.same_length && n1 !== null) say += "At stage 1 the next character is identical in both (0.00): it is the same letter in the same place. At stage 2, the first look back, it jumps to " + n1.toFixed(2) + ": the change has been carried onto the next character. ";
  if (far !== null) say += "By the end of the sentence (" + (J - 1) + " characters on) the last stage holds them " + far.toFixed(2) + " apart.";
  let bj = -1, bv = 0; for (let j = 3; j < J; j++) if (p.state[K - 1][j] > bv) { bv = p.state[K - 1][j]; bj = j; }
  if (bj > 0 && bv > 0.15 && bv > 2 * p.state[6][bj]) say += " Look further right in the last two stages: the difference comes back (" + bv.toFixed(2) + ") at \u201c" + p.after.slice(Math.max(0, bj - 2), bj + 1).replace(/ /g, "\u00b7") + "\u201d, " + bj + " characters after the change, having been near zero there through the middle stages (" + p.state[6][bj].toFixed(2) + " at stage 7).";
  $("pv-say").textContent = say;
  $("pv-honest").textContent = (p.same_length ? "" : "The two words differ in length here, so every later character also sits one or more places along: a second change mixed into the first. Pairs of equal length are the clean ones. ") + "Before the changed word the model holds the two versions identically, exactly (checked for every pair). The sorts above are descriptions of what differs, from a word tagger; they are not yet labelled by kind of change. One small model, real story openings, seen by looking and not tested.";
  $("pv-stage").value = PV.k;
}
function wakeInit() {
  PV.sort = PSORTS.includes("name") ? "name" : PSORTS[0];
  const first = pvList().findIndex(p => p.same_length && p.state && p.state[0].length > 6); PV.idx = Math.max(0, first);
  PSORTS.forEach(sn => { const b = document.createElement("button"); b.className = "chip"; b.id = "pvs-" + sn.replace(/\W+/g, "-"); b.textContent = SORT_SAY[sn] || sn; b.setAttribute("aria-pressed", sn === PV.sort);
    b.onclick = () => { PV.sort = sn; const f = pvList().findIndex(p => p.same_length && p.state && p.state[0].length > 4); PV.idx = Math.max(0, f); $("pv-sorts").querySelectorAll("button").forEach(x => x.setAttribute("aria-pressed", x === b)); wake(); }; $("pv-sorts").appendChild(b); });
  $("pv-prev").onclick = () => { const n = pvList().length; PV.idx = (PV.idx + n - 1) % n; wake(); };
  $("pv-next").onclick = () => { PV.idx = (PV.idx + 1) % pvList().length; wake(); };
  $("pv-ramp").style.background = "linear-gradient(to right," + [0, 1, 2, 3, 4, 5].map(i => css("--seq" + i)).join(",") + ")";
  const cv = $("wake"); let drag = null, last = 0;
  cv.onpointerdown = e => { drag = [e.clientX, e.clientY, PV.yaw, PV.pitch]; try { cv.setPointerCapture(e.pointerId); } catch (x) {} };
  cv.onpointerup = () => { drag = null; };
  cv.onpointermove = e => { if (drag) { PV.yaw = drag[2] + (e.clientX - drag[0]) * 0.008; PV.pitch = Math.max(0.15, Math.min(1.35, drag[3] + (e.clientY - drag[1]) * 0.006)); wake(); } };
  $("pv-stage").oninput = e => { PV.k = +e.target.value; PV.play = false; $("pv-play").setAttribute("aria-pressed", "false"); wake(); };
  function tick(ts) { if (!PV.play) return; if (last) PV.k += (ts - last) / 900; last = ts; if (PV.k >= 12) { PV.k = 12; PV.play = false; $("pv-play").setAttribute("aria-pressed", "false"); } wake(); if (PV.play) requestAnimationFrame(tick); }
  $("pv-play").onclick = () => { PV.play = !PV.play; $("pv-play").setAttribute("aria-pressed", PV.play); if (PV.play) { PV.k = 0; last = 0; requestAnimationFrame(tick); } };
  try { addEventListener("resize", wake); } catch (x) {}
  wake();
}
// ---------------------------------------------------------------- the shape of what the model holds
const CS = { k: 0, yaw: 0.0, pitch: 0.0, mode: "place", play: false, hover: null, scr: [] };
const PLACE_COL = ["--ink3", "--s7", "--s1", "--s3", "--s2", "--s4"];
const STAGE_SAY = n => n === "embed" ? "the character itself, before any thinking" : n.startsWith("attn") ? "after looking back at earlier characters (round " + (+n.slice(-1) + 1) + ")" : "after rewriting what it holds (round " + (+n.slice(-1) + 1) + ")";
function dotColour(m, cols, seqc) {
  if (CS.mode === "place") return cols[m.place];
  const v = CS.mode === "pos" ? (m.pos - 13) / 242 : Math.max(0, Math.min(1, m.surprise / 6)), u = v * 5, i = Math.min(4, Math.floor(u));
  return mix(seqc[i], seqc[i + 1], u - i);
}
function film() {
  const cols = PLACE_COL.map(c => css(c)), seqc = [0, 1, 2, 3, 4, 5].map(i => hex(css("--seq" + i)));
  CL.stages.forEach((nm, k) => {
    const cv = $("film-" + k), g = cv.getContext("2d"), W = cv.width; g.clearRect(0, 0, W, W);
    for (let n = 0; n < CL.points.length; n += 2) { const a = CL.xyz[n][k]; g.fillStyle = dotColour(CL.points[n], cols, seqc); g.fillRect(W / 2 + a[0] * W / 5.2 - 1, W / 2 - a[1] * W / 5.2 - 1, 2, 2); }
    cv.style.outline = Math.round(CS.k) === k ? "2px solid var(--ink)" : "1px solid var(--line)";
  });
}
function cloud() {
  const cv = $("cloud"), dpr = Math.min(2, window.devicePixelRatio || 1), Wp = cv.clientWidth || 900, Hp = cv.clientHeight || 560;
  if (cv.width !== Math.round(Wp * dpr)) { cv.width = Math.round(Wp * dpr); cv.height = Math.round(Hp * dpr); }
  const g = cv.getContext("2d"); g.setTransform(dpr, 0, 0, dpr, 0, 0); g.clearRect(0, 0, Wp, Hp);
  const k0 = Math.min(11, Math.floor(CS.k)), t = CS.k - k0, e = t * t * (3 - 2 * t);
  const cy = Math.cos(CS.yaw), sy = Math.sin(CS.yaw), cp = Math.cos(CS.pitch), sp = Math.sin(CS.pitch), sc = Math.min(Wp, Hp) / 5.0;
  const cols = PLACE_COL.map(c => css(c)), seqc = [0, 1, 2, 3, 4, 5].map(i => hex(css("--seq" + i)));
  const pts = [];
  for (let n = 0; n < CL.points.length; n++) {
    const a = CL.xyz[n][k0], b = CL.xyz[n][k0 + 1], x = a[0] + (b[0] - a[0]) * e, y = a[1] + (b[1] - a[1]) * e, z = a[2] + (b[2] - a[2]) * e;
    const xr = x * cy - z * sy, zr = x * sy + z * cy;
    pts.push([Wp / 2 + sc * xr, Hp / 2 - sc * (y * cp - zr * sp), zr * cp + y * sp, n]);
  }
  pts.sort((p, q) => p[2] - q[2]);
  CS.scr = pts;
  for (const p of pts) { g.fillStyle = dotColour(CL.points[p[3]], cols, seqc); g.globalAlpha = 0.85; g.beginPath(); g.arc(p[0], p[1], 2.8 + Math.max(-0.8, Math.min(1.4, p[2] * 0.5)), 0, 6.2832); g.fill(); }
  g.globalAlpha = 1;
  if (CS.hover != null) { const p = pts.find(q => q[3] === CS.hover); if (p) { g.beginPath(); g.arc(p[0], p[1], 7, 0, 6.2832); g.strokeStyle = css("--ink"); g.lineWidth = 2; g.stroke(); } }
  const kn = Math.round(CS.k), name = CL.stages[kn], sep = CL.separation.place_in_word_3d[kn], chr = CL.separation.which_character_3d[kn], clk = CL.clock[kn], room = CL.room_in_use[kn];
  let say = "Stage " + (kn + 1) + " of 13: " + STAGE_SAY(name) + ". ";
  if (CS.hover != null) {
    const m = CL.points[CS.hover], story = CL.points.filter(q => q.story === m.story), at = story.indexOf(m), txt = story.map(q => q.byte === 10 ? "↵" : String.fromCharCode(q.byte)).join("");
    say = "This dot is the “" + (m.byte === 32 ? "space" : m.byte === 10 ? "line break" : String.fromCharCode(m.byte)) + "” in: …" + txt.slice(Math.max(0, at - 14), at) + "[" + txt[at] + "]" + txt.slice(at + 1, at + 12) + "… It is character " + m.pos + " of its story (" + CL.places[m.place] + ").";
  } else if (kn === 0) say += "Stripes: the dots line up by which character they are (" + Math.round(chr * 100) + "% of what you can see), with spaces off on their own.";
  else if (clk > 0.75) say += "A ring. It is a clock: going round the ring is going through the story, character by character (the match is " + clk.toFixed(2) + " out of 1). The widest thing the model is holding here is where it is in the text, not what the text says. Colour by position to see it. This is not a discovery: the model is handed its position as a signal at the start, and such signals are known to form a ring. What it shows is that the loud part of the state is not the part that carries meaning.";
  else say += "The clock is gone from view (" + clk.toFixed(2) + ") and the dots have gathered by where each character sits in its word (" + Math.round(sep * 100) + "% of what you can see): spaces, first letters, the insides of words, last letters.";
  $("c-say").textContent = say;
  $("c-honest").textContent = "The model’s state has " + CL.width + " directions to use. At this stage it is really using about " + Math.round(room.effective) + " of them (" + room.for_90_percent + " hold nine tenths of the spread). This picture shows the widest three, which keep " + Math.round(CL.share_of_spread_kept_by_the_projection[kn] * 100) + "%. Eight stories, one small model; seen by looking, not tested.";
  $("c-stage").value = CS.k;
  $("c-key").innerHTML = CS.mode === "place" ? CL.places.map((n, i) => "<span><i style='background:var(" + PLACE_COL[i] + ");width:10px;height:10px;border-radius:50%'></i>" + esc(n) + "</span>").join("")
    : CS.mode === "pos" ? "<span><i style='background:var(--seq1);width:10px;height:10px;border-radius:50%'></i>start of the story</span><span><i style='background:var(--seq5);width:10px;height:10px;border-radius:50%'></i>end of the story</span>"
    : "<span><i style='background:var(--seq1);width:10px;height:10px;border-radius:50%'></i>next character easy to guess</span><span><i style='background:var(--seq5);width:10px;height:10px;border-radius:50%'></i>hard to guess</span>";
  CL.stages.forEach((nm, k) => { const f = $("film-" + k); if (f) f.style.outline = kn === k ? "2px solid var(--ink)" : "1px solid var(--line)"; });
}
function cloudInit() {
  const cv = $("cloud"); let drag = null, last = 0;
  CL.stages.forEach((nm, k) => { const c = document.createElement("canvas"); c.width = 120; c.height = 120; c.id = "film-" + k; c.title = "stage " + (k + 1); c.style.cssText = "width:76px;height:76px;cursor:pointer;border-radius:3px;background:var(--bg)"; c.onclick = () => { CS.k = k; CS.play = false; $("c-play").setAttribute("aria-pressed", "false"); cloud(); }; $("c-film").appendChild(c); });
  cv.onpointerdown = e => { drag = [e.clientX, e.clientY, CS.yaw, CS.pitch]; try { cv.setPointerCapture(e.pointerId); } catch (x) {} };
  cv.onpointerup = () => { drag = null; };
  cv.onpointermove = e => {
    if (drag) { CS.yaw = drag[2] + (e.clientX - drag[0]) * 0.008; CS.pitch = Math.max(-1.3, Math.min(1.3, drag[3] + (e.clientY - drag[1]) * 0.006)); CS.hover = null; cloud(); return; }
    const r = cv.getBoundingClientRect(), x = e.clientX - r.left, y = e.clientY - r.top; let best = null, bd = 81;
    for (const p of CS.scr) { const d = (p[0] - x) * (p[0] - x) + (p[1] - y) * (p[1] - y); if (d <= bd) { bd = d; best = p[3]; } }
    if (best !== CS.hover) { CS.hover = best; cloud(); }
  };
  cv.onpointerleave = () => { if (CS.hover != null) { CS.hover = null; cloud(); } };
  $("c-stage").oninput = e => { CS.k = +e.target.value; CS.play = false; $("c-play").setAttribute("aria-pressed", "false"); cloud(); };
  for (const m of ["place", "pos", "surprise"]) $("c-" + m).onclick = () => { CS.mode = m; for (const q of ["place", "pos", "surprise"]) $("c-" + q).setAttribute("aria-pressed", q === m); film(); cloud(); };
  function tick(ts) { if (!CS.play) return; if (last) CS.k += (ts - last) / 1600; last = ts; if (CS.k >= 12) { CS.k = 12; CS.play = false; $("c-play").setAttribute("aria-pressed", "false"); } if (!drag) cloud(); if (CS.play) requestAnimationFrame(tick); }
  $("c-play").onclick = () => { CS.play = !CS.play; $("c-play").setAttribute("aria-pressed", CS.play); if (CS.play) { if (CS.k >= 12) CS.k = 0; last = 0; requestAnimationFrame(tick); } };
  try { addEventListener("resize", cloud); } catch (x) {}
  film(); cloud();
}
// ---------------------------------------------------------------- the model in three dimensions
const FOUND = [
  { t: "It takes in one character and its place in the story", s: "The first stage holds only the character (a letter, a space, a punctuation mark) and where it sits. Everything after is built on top of it.", st: [0], how: "how the model is built" },
  { t: "It looks back, then recognises the word", s: "The first looking-back stage mixes in what came before: only 59% of how its output varies is down to which word it is. The next stage recognises the word from its last few letters and writes it consistently (84%).", st: [1, 2], how: "tested, and it held" },
  { t: "A first tested wire, and it is a thin one", s: "One reader in the first looking-back stage reads the character just before. It feeds the word lookup a little: without it the lookup keeps 0.83 of 0.87. No other reader hurts it as much, and two of them, taken out, make the lookup sharper. The lookup does not depend on any single reader.", st: [1, 2], how: "tested, and it held, by a small margin" },
  { t: "The same pattern repeats in the next pair", s: "In the next pair of stages the figures are 45%, then 80%. Whether the word is rebuilt at every stage, and never stored once, is suspected and not tested: only these first two pairs were measured.", st: [3, 4], how: "suspected" },
  { t: "Early on there is a lever for “adjective or noun next”", s: "In the first three rewriting stages, pushing along one direction changes which kind of word comes next, whatever its first letter (9 of 9 cases, on stories the model had never seen). Whether the model pulls that lever itself while reading is not yet known.", st: [2, 4, 6], how: "tested, and it held" },
  { t: "Late on, the same push is just a guess at the next letter", s: "By the last stage, the direction that makes adjectives likelier is almost purely a forecast of the next letter: pushing “next letter is l” raises every word starting with l, adjective or not. That it comes from the early lever was not tested.", st: [8, 10, 12], how: "tested, and it held" },
  { t: "The stage that adds most is not the one that matters most", s: "The last stage adds the most, but take it out and the model loses little (0.5). Take out the first rewriting stage, which adds far less, and it loses eight times as much (4.1); the first looking-back stage, 2.2. Everything later is built on those two. Switch the height to “what breaks without it” to see it.", st: [1, 2, 12], how: "measured, with no prediction made first" },
  { t: "What an outside reader finds is not what the model uses", s: "A reader trained from outside finds “an adjective comes next” along one direction. The direction that actually moves it is as unrelated as two random directions (82 to 90 degrees), and removing the reader’s direction costs the model nothing.", st: [2, 4, 6, 8, 10, 12], how: "measured as planned, no prediction staked" },
];
const S3 = { item: 0, win: 13, yaw: -0.62, pitch: 0.52, hi: null, spin: false, hover: null, tops: [], mode: "adds" };
const DASH = [[], [7, 4], [2, 4], [9, 3, 2, 3]];
const NAMED = { "attn0/3": "the character just before", "attn2/3": "the first letter of the current word", "attn3/1": "the space before the current word, where the word began" };
const COST = n => (TR.knock_out_cost[n] ? TR.knock_out_cost[n].realised : null), COSTMAX = Math.max(...Object.values(TR.knock_out_cost).map(v => v.realised));
const NAMES = { embed: "takes in the letter", attn: "looks back", mlp: "rewrites" };
function stageWord(n) { return n === "embed" ? "stage 1, the letter" : (n.startsWith("attn") ? "looking back, round " : "rewriting, round ") + (n === "embed" ? "" : (+n.slice(-1) + 1)); }
function scene() {
  const cv = $("scene"), it = TR.items[S3.item], dpr = Math.min(2, window.devicePixelRatio || 1), Wp = cv.clientWidth || 900, Hp = cv.clientHeight || 540;
  if (cv.width !== Math.round(Wp * dpr)) { cv.width = Math.round(Wp * dpr); cv.height = Math.round(Hp * dpr); }
  const g = cv.getContext("2d"); g.setTransform(dpr, 0, 0, dpr, 0, 0); g.clearRect(0, 0, Wp, Hp);
  const NX = Wp < 560 ? 28 : 44, K = it.writes.length, mx = Math.max(...it.write_norm.map(r => Math.max(...r)));
  const win = Math.min(S3.win, it.positions - NX), dx = 1, dz = 2.3, HH = 9;
  const cy0 = Math.cos(S3.yaw), sy0 = Math.sin(S3.yaw), cp = Math.cos(S3.pitch), sp = Math.sin(S3.pitch), sc = Math.min(Wp / (NX * 1.25), Hp / 34);
  const P = (x, y, z) => { const xr = x * cy0 - z * sy0, zr = x * sy0 + z * cy0; return [Wp / 2 + sc * xr, Hp * 0.62 - sc * (y * cp - zr * sp), zr * cp + y * sp]; };
  const col = { embed: hex(css("--ink3")), attn: hex(css("--s1")), mlp: hex(css("--s2")) }, bg = hex(css("--bg")), ink = hex(css("--ink"));
  const bars = [];
  for (let k = 0; k < K; k++) for (let i = 0; i < NX; i++) {
    const cst = COST(it.writes[k]);
    const x = (i - NX / 2) * dx, z = (k - K / 2) * dz, h = S3.mode === "adds" ? Math.max(0.05, HH * it.write_norm[win + i][k] / mx) : (cst === null ? 0.05 : Math.max(0.05, HH * cst / COSTMAX));
    bars.push({ k, i, x, z, h, d: P(x + .4, h / 2, z + .8)[2] });
  }
  bars.sort((a, b) => b.d - a.d);
  S3.tops = [];
  const poly = (pts, fill) => { g.beginPath(); pts.forEach((p, j) => j ? g.lineTo(p[0], p[1]) : g.moveTo(p[0], p[1])); g.closePath(); g.fillStyle = fill; g.fill(); };
  for (const b of bars) {
    const n = it.writes[b.k], kind = n === "embed" ? "embed" : n.slice(0, -1), base = col[kind];
    const dim = S3.hi && !S3.hi.includes(b.k), hot = S3.hover && S3.hover.k === b.k && S3.hover.i === b.i;
    const c = dim ? base.map((v, j) => Math.round(v + (bg[j] - v) * 0.8)) : base;
    const x0 = b.x, x1 = b.x + .8, z0 = b.z, z1 = b.z + 1.5, h = b.h;
    const A = P(x0, 0, z0), B = P(x1, 0, z0), C = P(x1, 0, z1), D = P(x0, 0, z1), E = P(x0, h, z0), F = P(x1, h, z0), G = P(x1, h, z1), H = P(x0, h, z1);
    const shade = (t, toward) => mix(c, toward, t);
    if (sy0 < 0) poly([B, C, G, F], shade(.38, bg)); else poly([A, D, H, E], shade(.38, bg));
    poly(cy0 > 0 ? [A, B, F, E] : [D, C, G, H], shade(.22, bg));
    poly([E, F, G, H], hot ? mix(c, ink, .45) : shade(0, bg));
    S3.tops.push({ k: b.k, i: b.i, pts: [E, F, G, H, A, B, C, D] });
  }
  let arcsOut = 0, arcsIn = 0;
  if (S3.pick != null && S3.pick >= win && S3.pick < win + NX) {
    const pi = S3.pick - win; g.lineCap = "round";
    for (let k = 0; k < K; k++) {
      const n = it.writes[k]; if (!n.startsWith("attn")) continue;
      const z = (k - K / 2) * dz + .75, hk = S3.mode === "adds" ? HH * it.write_norm[S3.pick][k] / mx : 0.3;
      const row = it.each_head_looks_at[n][S3.pick];
      row.forEach(([j, wgt], hd) => {
        if (j === S3.pick) return;
        if (j < win) { arcsOut++; return; }
        arcsIn++;
        const named = NAMED[n + "/" + hd], rest = j < 13;
        const a = P((pi - NX / 2) * dx + .4, hk, z), b = P((j - win - NX / 2) * dx + .4, 0.2, z), m = P(((pi + j - win) / 2 - NX / 2) * dx + .4, hk + 1.6 + hd * 0.7 + Math.abs(pi - (j - win)) * 0.2, z);
        g.beginPath(); g.moveTo(a[0], a[1]); g.quadraticCurveTo(m[0], m[1], b[0], b[1]);
        g.setLineDash(DASH[hd]); g.strokeStyle = named ? css("--ink") : css("--ink2"); g.globalAlpha = rest ? 0.18 : Math.min(1, 0.3 + wgt * 1.4); g.lineWidth = named ? 2.6 : 1.3; g.stroke(); g.globalAlpha = 1; g.setLineDash([]);
      });
    }
  }
  S3.arcNote = S3.pick == null ? "" : " Each looking-back stage has four readers; each line is one reader and the earlier character it draws on most (a different dash for each). Three readers have been seen, while exploring, to do something nameable, and are drawn heavier: in the first looking-back stage one reads the character just before; in the third, one reads the first letter of the current word; in the fourth, one reads the space where the word began. Where a line points says where a stage reads from, not how much that matters." + (arcsOut ? " " + arcsOut + " of them reach back before this window." : "");
  g.font = "11px " + css("--mono"); g.fillStyle = css("--ink2"); g.textBaseline = "middle";
  const front = cy0 > 0 ? -K / 2 * dz - 1.2 : (K / 2) * dz + 0.6;
  g.textAlign = "center";
  for (let i = 0; i < NX; i++) { const c = it.bytes[win + i], ch = c === 10 ? "↵" : c === 32 ? "·" : (c > 32 && c < 127 ? String.fromCharCode(c) : "□"), p = P((i - NX / 2) * dx + .4, 0, front); g.fillStyle = (S3.hover && S3.hover.i === i) || S3.pick === win + i ? css("--ink") : css("--ink2"); if (S3.pick === win + i) { g.font = "bold 13px " + css("--mono"); } g.fillText(ch, p[0], p[1]); g.font = "11px " + css("--mono"); }
  const side = sy0 < 0 ? -NX / 2 * dx - 1.4 : NX / 2 * dx + 1.4; g.textAlign = sy0 < 0 ? "right" : "left";
  for (let k = 0; k < K; k++) { const n = it.writes[k], p = P(side, 0, (k - K / 2) * dz + .75); g.fillStyle = S3.hi && S3.hi.includes(k) ? css("--ink") : css("--ink3"); g.fillText((k + 1) + "  " + (n === "embed" ? NAMES.embed : NAMES[n.slice(0, -1)]), p[0], p[1]); }
  const share = it.write_norm.reduce((a, r) => a + r[K - 1] * r[K - 1], 0) / it.write_norm.reduce((a, r) => a + r.reduce((u, v) => u + v * v, 0), 0);
  FOUND[5].live = "In this story the final stage accounts for " + Math.round(share * 100) + "% of the squared size of all the writes.";
  const f = S3.sel != null ? FOUND[S3.sel] : null, hv = S3.hover;
  $("s3say").textContent = hv && S3.mode !== "adds" ? "Stage " + (hv.k + 1) + " (" + stageWord(it.writes[hv.k]) + "): " + (COST(it.writes[hv.k]) === null ? "it was not taken out in the experiment." : "without it the model is " + COST(it.writes[hv.k]).toFixed(2) + " worse at the next letter. Reading its contribution off directly would have predicted " + TR.knock_out_cost[it.writes[hv.k]].direct.toFixed(2) + ".")
    : hv ? "Letter “" + (it.bytes[win + hv.i] === 32 ? "space" : it.bytes[win + hv.i] === 10 ? "new line" : String.fromCharCode(it.bytes[win + hv.i])) + "”, stage " + (hv.k + 1) + " (" + stageWord(it.writes[hv.k]) + "): it added " + it.write_norm[win + hv.i][hv.k].toFixed(1) + ", against " + mx.toFixed(1) + " for the largest block in this story."
    : f ? f.s + (f.live ? " " + f.live : "")
    : S3.pick != null ? "Picked: character " + S3.pick + ", \u201c" + (it.bytes[S3.pick] === 32 ? "space" : it.bytes[S3.pick] === 10 ? "new line" : String.fromCharCode(it.bytes[S3.pick])) + "\u201d." + S3.arcNote + " Click it again to clear."
    : S3.mode === "adds" ? "Letters run left to right, the 13 stages run front to back, and the story is read one letter at a time. Tall blocks are where a stage added a lot. The back row, the last stage, towers over the rest. That does not make it the stage that matters most: switch the height to see."
    : "Now the height is what breaks when a stage is taken out, measured for each stage as a whole (so it is the same along the story). The two stages at the front carry almost everything: without the first rewriting stage the model is 4.1 worse at the next letter, against 0.5 for the last stage. The first stage has no bar because it was not taken out.";
}
function s3init() {
  const cv = $("scene");
  TR.items.forEach((it, i) => { const b = document.createElement("button"); b.className = "chip"; b.id = "s3item-" + i; b.textContent = "Story " + (i + 1); b.setAttribute("aria-pressed", i === 0); b.onclick = () => { S3.item = i; $("s3items").querySelectorAll("button").forEach((x, j) => x.setAttribute("aria-pressed", j === i)); scene(); }; $("s3items").appendChild(b); });
  FOUND.forEach((f, i) => { const b = document.createElement("button"); b.id = "found-" + i; b.setAttribute("aria-pressed", "false"); b.innerHTML = "<em>" + esc(f.how) + "</em><b>" + esc(f.t) + "</b>"; b.onclick = () => { const on = S3.sel === i; S3.sel = on ? null : i; S3.hi = on ? null : f.st; $("s3found").querySelectorAll("button").forEach((x, j) => x.setAttribute("aria-pressed", !on && j === i)); scene(); }; $("s3found").appendChild(b); });
  $("s3win").oninput = e => { S3.win = +e.target.value; scene(); };
  for (const m of ["adds", "cost"]) $("s3h-" + m).onclick = () => { S3.mode = m; $("s3h-adds").setAttribute("aria-pressed", m === "adds"); $("s3h-cost").setAttribute("aria-pressed", m === "cost"); scene(); };
  let drag = null;
  cv.onpointerdown = e => { drag = [e.clientX, e.clientY, S3.yaw, S3.pitch]; try { cv.setPointerCapture(e.pointerId); } catch (x) {} };
  cv.onpointerup = e => {
    const moved = drag && (Math.abs(e.clientX - drag[0]) + Math.abs(e.clientY - drag[1]) > 5); drag = null;
    if (!moved && S3.hover) { const it = TR.items[S3.item], NXn = (cv.clientWidth || 900) < 560 ? 28 : 44, w0 = Math.min(S3.win, it.positions - NXn), p = w0 + S3.hover.i; S3.pick = S3.pick === p ? null : p; S3.sel = null; S3.hi = null; $("s3found").querySelectorAll("button").forEach(x => x.setAttribute("aria-pressed", "false")); S3.hover = null; scene(); }
  };
  cv.onpointermove = e => {
    if (drag) { S3.yaw = drag[2] + (e.clientX - drag[0]) * 0.008; S3.pitch = Math.max(0.12, Math.min(1.35, drag[3] + (e.clientY - drag[1]) * 0.006)); S3.hover = null; scene(); return; }
    const r = cv.getBoundingClientRect(), x = e.clientX - r.left, y = e.clientY - r.top; let hit = null;
    const inside = (q, px, py) => { let c = false; for (let i = 0, j = q.length - 1; i < q.length; j = i++) if (((q[i][1] > py) !== (q[j][1] > py)) && (px < (q[j][0] - q[i][0]) * (py - q[i][1]) / (q[j][1] - q[i][1]) + q[i][0])) c = !c; return c; };
    for (let n = S3.tops.length - 1; n >= 0 && !hit; n--) { const t = S3.tops[n], xs = t.pts.map(p => p[0]), ys = t.pts.map(p => p[1]); if (x >= Math.min(...xs) && x <= Math.max(...xs) && y >= Math.min(...ys) && y <= Math.max(...ys) && (inside(t.pts.slice(0, 4), x, y) || inside([t.pts[0], t.pts[1], t.pts[5], t.pts[4]], x, y) || inside([t.pts[1], t.pts[2], t.pts[6], t.pts[5]], x, y) || inside([t.pts[3], t.pts[0], t.pts[4], t.pts[7]], x, y) || inside([t.pts[2], t.pts[3], t.pts[7], t.pts[6]], x, y))) hit = { k: t.k, i: t.i }; }
    if ((hit && (!S3.hover || hit.k !== S3.hover.k || hit.i !== S3.hover.i)) || (!hit && S3.hover)) { S3.hover = hit; scene(); }
  };
  cv.onpointerleave = () => { if (S3.hover) { S3.hover = null; scene(); } };
  let still = false; try { still = matchMedia("(prefers-reduced-motion: reduce)").matches; } catch (x) {}
  $("s3spin").hidden = still;
  $("s3spin").onclick = () => { S3.spin = !S3.spin; $("s3spin").setAttribute("aria-pressed", S3.spin); if (S3.spin) tick(); };
  function tick() { if (!S3.spin) return; if (!drag) { S3.yaw += 0.004; scene(); } requestAnimationFrame(tick); }
  try { addEventListener("resize", scene); } catch (x) {}
  scene();
}
let tr = { item: 0, pos: Math.max(0, TR.items[0].fitted_site.indexOf(true)) };
function passView() {
  const it = TR.items[tr.item], P = it.positions;
  for (const b of $("items").querySelectorAll("button")) b.setAttribute("aria-pressed", +b.dataset.i === tr.item);
  const seq = v => { const t = Math.max(0, Math.min(1, v / 6)) * 5, i = Math.min(4, Math.floor(t)); return mix(hex(css("--seq" + i)), hex(css("--seq" + (i + 1))), t - i); };
  let h = "";
  for (let p = 0; p < P; p++) {
    const c = it.bytes[p], ch = c === 10 ? "↵" : c === 32 ? "·" : (c > 32 && c < 127 ? String.fromCharCode(c) : "□");
    const lp = p < it.next_byte_logprob_true.length ? -it.next_byte_logprob_true[p] : 0;
    h += "<button data-p='" + p + "' class='" + (p === tr.pos ? "cur" : "") + "' style='" + (it.fitted_site[p] ? "outline:2px solid var(--s2);outline-offset:-2px;" : "") + "background:" + seq(lp) + ";color:" + (lp > 3 ? "var(--bg)" : "var(--ink)") + "' title='byte " + p + ": the next byte cost " + lp.toFixed(2) + " nats'>" + esc(ch) + "</button>";
  }
  $("bytes").innerHTML = h;
  $("bytes").querySelectorAll("button").forEach(b => { b.onclick = () => { tr.pos = +b.dataset.p; passView(); }; });
  const c = it.bytes[tr.pos];
  $("wtitle").textContent = "At byte " + tr.pos + " (" + (c === 10 ? "a new line" : c === 32 ? "a space" : "“" + String.fromCharCode(c) + "”") + ")";
  const w = it.write_norm[tr.pos], mx = Math.max(...it.write_norm.map(r => Math.max(...r)));
  let s = "";
  it.writes.forEach((n, k) => {
    const y = 8 + k * 22, len = 300 * w[k] / mx;
    s += "<text x='52' y='" + (y + 12) + "' text-anchor='end'>" + n + "</text><rect x='58' y='" + y + "' width='" + Math.max(1, len) + "' height='16' rx='2' fill='" + (n.startsWith("mlp") ? "var(--ink2)" : n === "embed" ? "var(--ink3)" : "var(--seq3)") + "'><title>" + n + ": " + w[k] + "</title></rect><text x='" + (64 + len) + "' y='" + (y + 12) + "'>" + w[k].toFixed(2) + "</text>";
  });
  $("wbars").innerHTML = s;
  $("wbars").setAttribute("aria-label", "length of each block's write at byte " + tr.pos);
  const R = it.readings_after_mlp, Ls = Object.keys(R.probe).sort(), names = ["probe", "write", "resid", "next_byte_forecast"], cols = ["--s1", "--s2", "--s3", "--ink3"];
  if (!it.fitted_site[tr.pos]) {
    $("rlines").innerHTML = "<text x='210' y='110' text-anchor='middle'>These readings do not apply at this byte.</text><text x='210' y='130' text-anchor='middle'>Click an outlined byte in the text above.</text>";
  } else {
    let lo = -1, hi = 1;
    for (const n of names) for (const L of Ls) { const v = R[n][L][tr.pos]; if (v !== null) { lo = Math.min(lo, v); hi = Math.max(hi, v); } }
    const X = i => 60 + i * (330 / (Ls.length - 1)), Y = v => 215 - 195 * (v - lo) / (hi - lo);
    let q = "<line x1='60' x2='390' y1='" + Y(0) + "' y2='" + Y(0) + "' stroke='var(--line)'/><text x='54' y='" + (Y(0) + 3) + "' text-anchor='end'>0</text><text x='54' y='" + (Y(hi) + 3) + "' text-anchor='end'>" + hi.toFixed(1) + "</text><text x='54' y='" + (Y(lo) + 3) + "' text-anchor='end'>" + lo.toFixed(1) + "</text>";
    Ls.forEach((L, i) => { q += "<text x='" + X(i) + "' y='236' text-anchor='middle'>" + L + "</text>"; });
    names.forEach((n, j) => {
      const pts = Ls.map((L, i) => [X(i), R[n][L][tr.pos]]).filter(p => p[1] !== null);
      if (!pts.length) return;
      q += "<path d='" + pts.map((p, i) => (i ? "L" : "M") + p[0] + " " + Y(p[1])).join("") + "' fill='none' stroke='var(" + cols[j] + ")' stroke-width='2'/>";
      pts.forEach(p => { q += "<circle cx='" + p[0] + "' cy='" + Y(p[1]) + "' r='4' fill='var(" + cols[j] + ")' stroke='var(--panel)' stroke-width='2'><title>" + p[1] + " standard deviations</title></circle>"; });
    });
    $("rlines").innerHTML = q;
  }
  $("rlines").setAttribute("aria-label", "four readings after each rewriting stage at byte " + tr.pos);
  $("okline").innerHTML = "<b>Checked before it was drawn.</b> At each of the " + P + " bytes of this story the 13 writes, added in the model's own order, reproduce the state bit for bit; a story that failed would not be here. Logits rebuilt from the writes differ from the model's by at most " + it.c4_max_abs_logit_residual.toExponential(1) + ". " + esc(it.source) + ".";
}
TR.items.forEach((it, i) => { const b = document.createElement("button"); b.className = "chip"; b.dataset.i = i; b.id = "item-" + i; b.textContent = "story " + (i + 1); b.onclick = () => { tr.item = i; tr.pos = Math.max(0, TR.items[i].fitted_site.indexOf(true)); passView(); }; $("items").appendChild(b); });
for (const [id, m] of Object.entries(T.models)) {
  const b = document.createElement("button"); b.className = "chip"; b.dataset.id = id; b.id = "model-" + id; b.textContent = m.title;
  b.onclick = () => { state.model = id; state.sel = null; render(); }; $("models").appendChild(b);
}
if (state.layer) state.model = layerOf(state.layer).model;
$("foot").textContent = "The model is GoldWorm v6, a small public model of 2.76 million parameters that reads children\u2019s stories one character at a time. Every finding here is about this one model; none is a claim about any other. The findings are the RATCHET session\u2019s experiments, stated here in plain words by the CIRISHolon session; \u201ctested, and it held\u201d means a prediction was written down before the experiment and came true. SCOPE-0, version " + SIDE.version + ": " + T.rows.length + " readings copied from result files of the RATCHET kindscope series (commit " + SIDE.ratchet + ") and of the earlier hand-over bundle. Nothing was measured or re-derived to draw this page. Which experiments feed each view was chosen by the CIRISHolon session and corrected once by the session that owns the instrument.";
wakeInit(); cloudInit(); s3init(); render(); passView();
try { matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => { wake(); film(); cloud(); scene(); render(); passView(); }); new MutationObserver(() => { wake(); film(); cloud(); scene(); render(); passView(); }).observe(document.documentElement, { attributes: true, attributeFilter: ["data-theme"] }); } catch (e) {}
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
    traces = open(os.path.join(d, "scope_traces.json"), encoding="utf-8").read().replace("</", "<\\/")
    shapes = open(os.path.join(d, "scope_shapes.json"), encoding="utf-8").read().replace("</", "<\\/")
    pairs = json.load(open(os.path.join(d, "scope_pairs.json"), encoding="utf-8"))
    for q in pairs["pairs"]:
        q.pop("write", None)          # the page draws the state's difference; the writes' stay in the table
    pairs = json.dumps(pairs, ensure_ascii=False, separators=(",", ":")).replace("</", "<\\/")
    open(out, "w", encoding="utf-8").write(PAGE.replace("__TABLE__", table).replace("__TRACES__", traces).replace("__SHAPES__", shapes).replace("__PAIRS__", pairs).replace("__SIDE__", side))
    print(out, os.path.getsize(out), "bytes")


if __name__ == "__main__":
    main()
