"""docs/atoms on holon-draw: the check, its pass conditions stated before its first run.

  python3 docs/holon-draw/checks/check_atoms.py [BEFORE_COMMIT]     (from the repo root)

BEFORE_COMMIT defaults to f206a4f (the stage and the curve inset drawn with Canvas 2D).
Images go to $CHECK_OUT (default ./check-out/atoms).

THE RUN: load; pump 20 frames; set the atoms slider to 12 (an input event, as a hand would);
pump 150 frames; press the pointer at the stage's centre and move it 60 px right and 30 px
down, pumping 5 frames at each of three steps (a grab if an atom is within the pick radius,
else nothing, the same in both); pump 5 more frames; screenshot, the pointer still down.

PASS CONDITIONS (written 2026-11-23, before the first run; never edited after a run):
  C1  the AFTER page raises no page error, and logs no console error other than the 404 of
      /favicon.ico at the server root (the request the browser makes by itself; see
      check_play.py's run 1).
  C2  #stage and #curve are each given a WebGL context and never a "2d" one.
  C3  no Canvas-2D drawing call is made on any canvas of the AFTER page.
  C4  in every pumped frame (the harness logs each one, the interaction's and the one after
      the overlays are hidden included), #stage and #curve each receive at least one WebGL draw call and
      exactly as many as draw.js's stats().drawCalls for that canvas's frame.
  C5  BEFORE and AFTER reach the SAME STATE: the text of every element with an id inside the
      inspector, and #stage-status, #clock and #atom-count-out, are equal (all of it is read
      from the wasm; with the virtual clock, the calibration and the frame intervals are the
      same numbers in both).
  C6  behaviour in AFTER: body[data-engine] is "ready", the energy and momentum gates read
      PASS or FAIL as BEFORE does (part of C5), and #atom-count-out reads 12.
  C7  a before/after pair of the stage and of the curve inset is saved with a pixel-
      difference summary (reported, not thresholded: atoms are 16-band disc stacks, dashes
      are cut segments, bond ends are flat).
"""
import json, os, sys
sys.dont_write_bytecode = True
here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
import harness

repo = os.path.abspath(os.path.join(here, "..", "..", ".."))
commit = sys.argv[1] if len(sys.argv) > 1 else "f206a4f"
out = os.environ.get("CHECK_OUT", os.path.join(os.getcwd(), "check-out", "atoms"))
PRE = 20


def extra(page):
    page.evaluate("""(() => { const el = document.getElementById('atom-count'); el.value = '12';
        el.dispatchEvent(new Event('input', { bubbles: true })); el.dispatchEvent(new Event('change', { bubbles: true })); })()""")
    page.evaluate("window.__pump(150)")
    box = page.locator("#stage").bounding_box()
    x, y = box["x"] + box["width"] / 2, box["y"] + box["height"] / 2
    page.mouse.move(x, y)
    page.mouse.down()
    for k in range(1, 4):
        page.mouse.move(x + 20 * k, y + 10 * k)
        page.evaluate("window.__pump(5)")
    page.evaluate("window.__pump(5)")
    page.locator("#curve").screenshot(path=os.path.join(out, page.__label + "-curve.png"))


check = {
    "cfg": {"seed": 20261123, "clockStep": 25},
    "ready": "document.body.dataset.engine === 'ready' && window.__queued() > 0",
    "state": """(() => {
        const o = {};
        for (const el of document.querySelectorAll('.inspector [id], #stage-status, #clock, #atom-count-out')) o[el.id] = el.textContent;
        o.__engine = document.body.dataset.engine;
        return o;
    })()""",
    "per_frame_js": "window.__atomsStages ? { stage: window.__atomsStages.stage.lastFrame, curve: window.__atomsStages.curve.lastFrame } : null",
    "hide_css": ".stage-topline, .aim-copy { visibility: hidden !important; }",
    "shot": "#stage",
    "viewport": {"width": 1280, "height": 900},
}

results = {}
os.makedirs(out, exist_ok=True)
before_root = harness.before_tree(repo, commit, out)
for label, root in (("before", before_root), ("after", os.path.join(repo, "docs"))):
    srv, port = harness.serve(root)
    try:
        def ex(page, label=label):
            page.__label = label
            extra(page)
        results[label] = harness.run_page(f"http://127.0.0.1:{port}/atoms/", check["cfg"], PRE, check["ready"], check["state"],
                                          check["hide_css"], check["shot"], os.path.join(out, f"{label}.png"), check["viewport"],
                                          ex, check["per_frame_js"])
    finally:
        srv.terminate()
A, B = results["after"], results["before"]
d_stage = harness.diff_summary(os.path.join(out, "before.png"), os.path.join(out, "after.png"), os.path.join(out, "diff.png"))
d_curve = harness.diff_summary(os.path.join(out, "before-curve.png"), os.path.join(out, "after-curve.png"), os.path.join(out, "diff-curve.png"))
d_page = harness.diff_summary(os.path.join(out, "before-page.png"), os.path.join(out, "after-page.png"), os.path.join(out, "diff-page.png"))

def ctx_ok(cid):
    cs = [c for c in A["contexts"] if c["canvas"].startswith(cid)]
    return bool(cs) and all(c["type"] in ("webgl2", "webgl") for c in cs)

def frame_ok(f):
    p = f["page"]
    return bool(p) and all(f["gl"].get(c, 0) > 0 and f["gl"].get(c, 0) == p[c]["drawCalls"] for c in ("stage", "curve"))

good = [f for f in A["per_frame"] if frame_ok(f)]
conds = [
    ("C1 no page error; no console error but the root /favicon.ico 404",
     not A["errors"] and all("/favicon.ico" in e for e in A["console_errors"]), A["errors"] + A["console_errors"]),
    ("C2 #stage and #curve have WebGL contexts and never a 2d one", ctx_ok("stage") and ctx_ok("curve"), A["contexts"]),
    ("C3 no Canvas-2D drawing call on any canvas", not A["draw2d_total"], A["draw2d_total"]),
    (f"C4 every frame's stage and curve draws are draw.js's ({len(good)}/{len(A['per_frame'])})",
     len(good) == len(A["per_frame"]) and len(good) > 0, A["per_frame"][:2]),
    ("C5 the same state before and after", A["state"] == B["state"],
     {k: (B["state"].get(k), A["state"].get(k)) for k in set(A["state"]) | set(B["state"]) if A["state"].get(k) != B["state"].get(k)}),
    ("C6 engine ready, 12 atoms", A["state"]["__engine"] == "ready" and A["state"].get("atom-count-out") == "12", A["state"]),
    ("C7 before/after pairs and diffs saved", "error" not in d_stage and "error" not in d_curve, [d_stage, d_curve]),
]
report = {"commit_before": commit, "out": out, "gpu": A["gpu"], "diff_stage": d_stage, "diff_curve": d_curve, "diff_page": d_page,
          "state_after": A["state"], "draws_last_frame_after": A["per_frame"][-1], "contexts_after": A["contexts"],
          "contexts_before": B["contexts"], "errors_before": B["errors"] + B["console_errors"], "warnings_after": A["console_warnings"]}
json.dump(report, open(os.path.join(out, "report.json"), "w"), indent=1)
print(json.dumps({k: report[k] for k in ("gpu", "diff_stage", "diff_curve", "diff_page", "draws_last_frame_after")}, indent=1))
sys.exit(harness.verdict(conds))
