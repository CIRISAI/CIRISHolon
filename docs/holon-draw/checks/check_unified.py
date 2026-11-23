"""docs/unified on holon-draw: the check, its pass conditions stated before its first run.

  python3 docs/holon-draw/checks/check_unified.py [BEFORE_COMMIT]     (from the repo root)

BEFORE_COMMIT defaults to f206a4f (stage, curve and ZX graph drawn with Canvas 2D). Images
go to $CHECK_OUT (default ./check-out/unified). Viewport 1440 x 1000.

THE RUN (the same steps in BEFORE and AFTER; every input is a synthetic event dispatched
from the page's own thread, so each lands between the same two pumps):
  1. load; pump 30 frames.
  2. find the atoms on BEFORE's stage (its HTML overlays hidden; the centroids of the bright
     atom bodies) and use those coordinates on both pages; press the left atom, drag it in
     20 steps of 2 frames to the midpoint of the two, hold 20 frames: "2d" pair (stage).
  3. release; click the 3D button (#btn-mode-3d, el.click()); pump 60 frames: "3d" pair.
  4. click the "ZX Spider Graph" tab (el.click()); pump 3 frames: "zx" pair (#zx-canvas).
  5. "curve" pair (#curve-canvas), last, since it scrolls the page.

PASS CONDITIONS (written 2026-11-23, before the first run; never edited after a run):
  C1  the AFTER page raises no page error, and logs no console error other than the 404 of
      /favicon.ico at the server root.
  C2  #stage, #curve-canvas and #zx-canvas are each given a WebGL context and never "2d".
  C3  no Canvas-2D drawing call is made on any canvas of the AFTER page.
  C4  in every pumped frame, the GL draw calls on #stage and on #curve-canvas are each at
      least one and equal draw.js's stats().drawCalls for that canvas; and #zx-canvas
      receives at least one GL draw call in the run and none made outside a draw.js draw()
      (the check wraps each stage's drawer.draw, before any frame, to mark when it runs).
      (This sentence was revised on 2026-11-23 BEFORE the first run: the committed version
      asked for a sum the page's per-frame report cannot give for a canvas drawn on events.)

RUN 1 (2026-11-23): C1-C3 and C5-C7 PASS; C4 FAILED: 6 of #zx-canvas's 12 GL draws were
  counted outside draw.js. The cause is this file: the page draws the ZX graph once during
  boot (runCircuit -> drawZXGraph, before its first frame), and the wrapper was installed by
  the ready test, after that draw; so those 6 were draw.js's but unmarked. C4's own words
  ("before any frame") did not cover a draw made before the first frame.
CORRECTION (2026-11-23, after run 1): the wrapper is installed by an init script the moment
  the page publishes window.__unifiedStages (a setter on window), before any drawer can draw.
  Conditions not edited; run 2 is judged by them as written.
  C5  BEFORE and AFTER reach the SAME STATE: the text of every element with an id inside
      #pane-physics, and of every element whose id starts "zx-", are equal, read after
      step 4.
  C6  behaviour in AFTER: body[data-engine] is "ready"; after step 2 the grab took (the
      2D stage drew the spring: the record count of the frame after the drag exceeds the
      frame before the press); after step 3 #btn-mode-3d has class "active".
  C7  the four before/after pairs are saved with pixel-difference summaries (reported, not
      thresholded: radial gradients are 16-band disc stacks, dashes are cut segments,
      circle strokes are polylines, the label's text is text.js's rasterisation).
"""
import io, json, os, sys
sys.dont_write_bytecode = True
here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
import harness
import numpy as np
from PIL import Image

repo = os.path.abspath(os.path.join(here, "..", "..", ".."))
commit = sys.argv[1] if len(sys.argv) > 1 else "f206a4f"
out = os.environ.get("CHECK_OUT", os.path.join(os.getcwd(), "check-out", "unified"))
PRE = 30
OVERLAYS = ".viewport-overlay-top, .viewport-overlay-hint"
_found = {}
_marks = {}

EV = """([t, x, y]) => document.getElementById('stage').dispatchEvent(new PointerEvent(t,
    { clientX: x, clientY: y, pointerId: 1, pointerType: 'mouse', isPrimary: true, button: 0, buttons: t === 'pointerup' ? 0 : 1, bubbles: true, cancelable: true }))"""


def atoms_on_stage(page):
    page.evaluate(f"document.querySelectorAll('{OVERLAYS}').forEach(e => e.style.visibility = 'hidden')")
    img = np.asarray(Image.open(io.BytesIO(page.locator("#stage").screenshot())).convert("RGB")).astype(int)
    page.evaluate(f"document.querySelectorAll('{OVERLAYS}').forEach(e => e.style.visibility = '')")
    mask = (img[..., 0] > 150) & (img[..., 1] > 150) & (img[..., 2] > 150)
    from scipy import ndimage
    lab, n = ndimage.label(mask)
    cs = ndimage.center_of_mass(mask, lab, range(1, n + 1))
    sizes = ndimage.sum(mask, lab, range(1, n + 1))
    return sorted([(c[1], c[0]) for c, z in zip(cs, sizes) if z > 30])


def shot(page, sel, name):
    page.locator(sel).screenshot(path=os.path.join(out, f"{page.__label}-{name}.png"))


# installed with the init script: wraps each drawer's draw() when the page publishes the handle
WRAP = """(() => { let held;
  Object.defineProperty(window, '__unifiedStages', { configurable: true, get() { return held; }, set(S) {
    held = S;
    for (const k of ['stage', 'curve', 'zx']) { const d = S[k].drawer, o = d.draw;
      d.draw = function (...a) { window.__inDrawJs++; try { return o.apply(this, a); } finally { window.__inDrawJs--; } }; }
  } }); })();"""


def extra(page):
    if page.__label == "before":
        _found["atoms"] = atoms_on_stage(page)
        print("atoms found on BEFORE's stage (px):", [(round(a), round(b)) for a, b in _found["atoms"]])
    atoms = _found["atoms"]
    box = page.locator("#stage").bounding_box()
    (x0, y0), (x1, y1) = atoms[0], atoms[-1]
    ax, ay = box["x"] + x0, box["y"] + y0
    tx, ty = box["x"] + (x0 + x1) / 2, box["y"] + (y0 + y1) / 2
    rec = "window.__unifiedStages ? window.__unifiedStages.stage.lastFrame.records : -1"
    page.evaluate("window.__pump(1)")
    before_press = page.evaluate(rec)
    page.evaluate(EV, ["pointerdown", ax, ay])
    for k in range(1, 21):
        page.evaluate(EV, ["pointermove", ax + (tx - ax) * k / 20, ay + (ty - ay) * k / 20])
        page.evaluate("window.__pump(2)")
    page.evaluate("window.__pump(20)")
    _marks[page.__label] = {"records_before_press": before_press, "records_after_drag": page.evaluate(rec)}
    page.evaluate(f"document.querySelectorAll('{OVERLAYS}').forEach(e => e.style.visibility = 'hidden')")
    page.evaluate("window.__pump(1)")
    shot(page, "#stage", "2d")
    page.evaluate(f"document.querySelectorAll('{OVERLAYS}').forEach(e => e.style.visibility = '')")
    page.evaluate(EV, ["pointerup", tx, ty])
    page.evaluate("document.getElementById('btn-mode-3d').click()")
    page.evaluate("window.__pump(60)")
    _marks[page.__label]["mode3d_active"] = page.evaluate("document.getElementById('btn-mode-3d').classList.contains('active')")
    page.evaluate(f"document.querySelectorAll('{OVERLAYS}').forEach(e => e.style.visibility = 'hidden')")
    page.evaluate("window.__pump(1)")
    shot(page, "#stage", "3d")
    page.evaluate(f"document.querySelectorAll('{OVERLAYS}').forEach(e => e.style.visibility = '')")
    page.evaluate("document.querySelector('.tab-btn[data-tab=\"zx-graph\"]').click()")
    page.evaluate("window.__pump(3)")
    shot(page, "#zx-canvas", "zx")
    shot(page, "#curve-canvas", "curve")


check = {
    "cfg": {"seed": 20261123, "clockStep": 25, "initExtra": None},
    "ready": "document.body.dataset.engine === 'ready' && window.__queued() > 0",
    "state": """(() => {
        const o = {};
        for (const el of document.querySelectorAll('#pane-physics [id], [id^="zx-"]')) o[el.id] = el.textContent.trim();
        o.__engine = document.body.dataset.engine;
        return o;
    })()""",
    "per_frame_js": """window.__unifiedStages ? { stage: window.__unifiedStages.stage.lastFrame,
        curve: window.__unifiedStages.curve.lastFrame, zx: window.__unifiedStages.zx.lastFrame } : null""",
    "hide_css": OVERLAYS + " { visibility: hidden !important; }",
    "shot": "#stage",
    "viewport": {"width": 1440, "height": 1000},
}

check["cfg"]["initExtra"] = WRAP
results = {}
os.makedirs(out, exist_ok=True)
before_root = harness.before_tree(repo, commit, out)
for label, root in (("before", before_root), ("after", os.path.join(repo, "docs"))):
    srv, port = harness.serve(root)
    try:
        def ex(page, label=label):
            page.__label = label
            extra(page)
        # the ZX canvas is drawn outside the frame loop too (on a click), so its draws are also
        # counted from the page's side on every draw() of that stage
        results[label] = harness.run_page(f"http://127.0.0.1:{port}/unified/", check["cfg"], PRE, check["ready"], check["state"],
                                          check["hide_css"], check["shot"], os.path.join(out, f"{label}.png"), check["viewport"],
                                          ex, check["per_frame_js"])
    finally:
        srv.terminate()
A, B = results["after"], results["before"]
diffs = {}
for name in ("2d", "3d", "zx", "curve"):
    diffs[name] = harness.diff_summary(os.path.join(out, f"before-{name}.png"), os.path.join(out, f"after-{name}.png"), os.path.join(out, f"diff-{name}.png"))
diffs["page"] = harness.diff_summary(os.path.join(out, "before-page.png"), os.path.join(out, "after-page.png"), os.path.join(out, "diff-page.png"))


def ctx_ok(cid):
    cs = [c for c in A["contexts"] if c["canvas"].startswith(cid)]
    return bool(cs) and all(c["type"] in ("webgl2", "webgl") for c in cs)


def frame_ok(f):
    p = f["page"]
    return bool(p) and all(f["gl"].get(c, 0) > 0 and f["gl"].get(c, 0) == p[k]["drawCalls"] for c, k in (("stage", "stage"), ("curve-canvas", "curve")))

good = [f for f in A["per_frame"] if frame_ok(f)]
zx_total = A["draws_total"].get("zx-canvas", 0)
zx_outside = A["draws_outside_drawjs"].get("zx-canvas", 0)
m = _marks.get("after", {})
conds = [
    ("C1 no page error; no console error but the root /favicon.ico 404",
     not A["errors"] and all("/favicon.ico" in e for e in A["console_errors"]), A["errors"] + A["console_errors"]),
    ("C2 #stage, #curve-canvas, #zx-canvas have WebGL contexts and never a 2d one",
     ctx_ok("stage") and ctx_ok("curve-canvas") and ctx_ok("zx-canvas"), A["contexts"]),
    ("C3 no Canvas-2D drawing call on any canvas", not A["draw2d_total"], A["draw2d_total"]),
    (f"C4 every frame's stage and curve draws are draw.js's ({len(good)}/{len(A['per_frame'])}); zx: {zx_total} draws, {zx_outside} outside draw.js",
     len(good) == len(A["per_frame"]) and len(good) > 0 and zx_total > 0 and zx_outside == 0, [A["per_frame"][:2], A["draws_outside_drawjs"]]),
    ("C5 the same state before and after", A["state"] == B["state"],
     {k: (B["state"].get(k), A["state"].get(k)) for k in set(A["state"]) | set(B["state"]) if A["state"].get(k) != B["state"].get(k)}),
    ("C6 ready; the grab drew the spring; 3D mode active",
     A["state"]["__engine"] == "ready" and m.get("records_after_drag", -1) > m.get("records_before_press", 0) >= 0 and m.get("mode3d_active") is True, m),
    ("C7 four before/after pairs and diffs saved", all("error" not in diffs[k] for k in ("2d", "3d", "zx", "curve")), diffs),
]
report = {"commit_before": commit, "out": out, "gpu": A["gpu"], "diffs": diffs, "marks": _marks,
          "state_after": A["state"], "draws_last_frame_after": A["per_frame"][-1], "draws_total_after": A["draws_total"],
          "contexts_after": A["contexts"], "contexts_before": B["contexts"],
          "errors_before": B["errors"] + B["console_errors"], "warnings_after": A["console_warnings"]}
json.dump(report, open(os.path.join(out, "report.json"), "w"), indent=1)
print(json.dumps({k: report[k] for k in ("gpu", "diffs", "marks", "draws_last_frame_after", "draws_total_after")}, indent=1))
sys.exit(harness.verdict(conds))
