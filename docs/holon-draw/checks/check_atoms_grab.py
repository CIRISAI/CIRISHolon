"""docs/atoms on holon-draw, the SECOND run: a grab, so the spring and the molecules are drawn.

  python3 docs/holon-draw/checks/check_atoms_grab.py [BEFORE_COMMIT]     (from the repo root)

check_atoms.py's run drew atoms and the wall only (no atom was within the pick radius of the
stage's centre, and no pair bonded), so the bond lines and the dashed spring were not in its
picture. This run puts them there. Images go to $CHECK_OUT (default ./check-out/atoms-grab).

THE RUN: load the default scene (2 atoms); pump 40 frames; in each page, find the atoms on
its own stage screenshot (the centroids of the atom-coloured pixels: the two pages are in
the same state, so they find the same atoms); press on the left one and drag it toward the
right one, to 3 bohr of it, in 30 steps of 3 frames; hold there 90 frames; screenshot with
the pointer still down.

PASS CONDITIONS (written 2026-11-23, before this file's first run; never edited after):
  C1..C7 as check_atoms.py states them, for this run (C6: body[data-engine] is "ready").
  C8  the spring is in the picture: in AFTER, #w-ext or #e-spring reads other than 0.000000
      (the grab took), the same as BEFORE (part of C5).
  Whether a molecule row (a bond line) is drawn is REPORTED (#bond-count), not a condition.
"""
import json, os, sys
sys.dont_write_bytecode = True
here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
import harness

repo = os.path.abspath(os.path.join(here, "..", "..", ".."))
commit = sys.argv[1] if len(sys.argv) > 1 else "f206a4f"
out = os.environ.get("CHECK_OUT", os.path.join(os.getcwd(), "check-out", "atoms-grab"))
PRE = 40


def atoms_on_stage(page):
    import numpy as np
    from PIL import Image
    import io
    img = np.asarray(Image.open(io.BytesIO(page.locator("#stage").screenshot())).convert("RGB")).astype(int)
    r, g, b = img[..., 0], img[..., 1], img[..., 2]
    mask = (g - r > 40) & (b - r > 25)          # the teal of hydrogen, not the paper or the wall
    from scipy import ndimage
    lab, n = ndimage.label(mask)
    cs = ndimage.center_of_mass(mask, lab, range(1, n + 1))
    sizes = ndimage.sum(mask, lab, range(1, n + 1))
    return sorted([(c[1], c[0]) for c, z in zip(cs, sizes) if z > 30])


def extra(page):
    atoms = atoms_on_stage(page)
    page.__found = atoms
    box = page.locator("#stage").bounding_box()
    (x0, y0), (x1, y1) = atoms[0], atoms[-1]
    s = box["width"] / 40.0                      # px per bohr: the world is 40 bohr wide
    ax, ay = box["x"] + x0, box["y"] + y0
    dx, dy = x1 - x0, y1 - y0
    d = (dx * dx + dy * dy) ** 0.5
    tx, ty = box["x"] + x1 - dx / d * 3 * s, box["y"] + y1 - dy / d * 3 * s
    page.mouse.move(ax, ay)
    page.mouse.down()
    for k in range(1, 31):
        page.mouse.move(ax + (tx - ax) * k / 30, ay + (ty - ay) * k / 30)
        page.evaluate("window.__pump(3)")
    page.evaluate("window.__pump(90)")
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
    ("C6 engine ready", A["state"]["__engine"] == "ready", A["state"]),
    ("C7 before/after pairs and diffs saved", "error" not in d_stage and "error" not in d_curve, [d_stage, d_curve]),
    ("C8 the grab took: w-ext or e-spring is not 0.000000", A["state"].get("w-ext") != "0.000000" or A["state"].get("e-spring") != "0.000000",
     {k: A["state"].get(k) for k in ("w-ext", "e-spring")}),
]
report = {"commit_before": commit, "out": out, "gpu": A["gpu"], "diff_stage": d_stage, "diff_curve": d_curve, "diff_page": d_page,
          "state_after": A["state"], "draws_last_frame_after": A["per_frame"][-1], "contexts_after": A["contexts"],
          "contexts_before": B["contexts"], "errors_before": B["errors"] + B["console_errors"], "warnings_after": A["console_warnings"],
          "bond_count_after": A["state"].get("bond-count"), "stage_status_after": A["state"].get("stage-status")}
json.dump(report, open(os.path.join(out, "report.json"), "w"), indent=1)
print(json.dumps({k: report[k] for k in ("gpu", "diff_stage", "diff_curve", "diff_page", "draws_last_frame_after", "bond_count_after", "stage_status_after")}, indent=1))
sys.exit(harness.verdict(conds))
