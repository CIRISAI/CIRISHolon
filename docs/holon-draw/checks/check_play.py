"""docs/play on holon-draw: the check, its pass conditions stated before its first run.

  python3 docs/holon-draw/checks/check_play.py [BEFORE_COMMIT]     (from the repo root)

BEFORE_COMMIT is the commit whose docs/play is "before" (default f206a4f, the last commit
on which the page drew its stage with Canvas 2D). Images go to $CHECK_OUT (default
./check-out/play), never into docs/.

PASS CONDITIONS (written 2026-11-23, before the first run; never edited after a run):
  C1  the AFTER page raises no page error and logs no console error.
  C2  the stage canvas (#stage) is given a WebGL context and is never asked for a "2d" one.
  C3  no Canvas-2D drawing call (fill, stroke, fillRect, strokeRect, fillText, strokeText,
      drawImage, putImageData) is made on any canvas of the AFTER page.
  C4  in each of the 60 pumped frames, the stage canvas receives at least one WebGL draw
      call, and exactly as many as draw.js's own stats().drawCalls for that frame: every
      draw on the stage is draw.js's.
  C5  BEFORE and AFTER reach the SAME STATE after 60 frames (seeded Math.random, the same
      virtual clock): the digest of every blob's position and every promoted molecule's
      atoms, and the HUD's non-timing fields (focus, molecules, blobs, engine), are equal.
  C6  the behaviour holds in AFTER: the HUD's engine field reads "live, dt ..." and the ring
      holds at least one real molecule.
  C7  a before/after pair of the stage (HUD, banner and legend hidden in both) is saved with
      a pixel-difference summary. The difference is REPORTED, not thresholded: the radial
      blobs are drawn as stacked discs (screen.js), so a difference is expected there.
  C8  docs/play/wall.mjs still passes.
"""
import json, os, subprocess, sys
sys.dont_write_bytecode = True
here = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, here)
import harness

repo = os.path.abspath(os.path.join(here, "..", "..", ".."))
commit = sys.argv[1] if len(sys.argv) > 1 else "f206a4f"
out = os.environ.get("CHECK_OUT", os.path.join(os.getcwd(), "check-out", "play"))
FRAMES = 60

check = {
    "cfg": {"seed": 20261123, "clockStep": 0.001},
    "ready": "(() => { const e = document.getElementById('h-engine'); return e && e.textContent !== 'booting' && window.__queued() > 0; })()",
    "state": """(() => {
        const t = window.__toy; let h = 0;
        const mix = (v) => { h = (Math.imul(h ^ Math.round(v * 1000), 2654435761) >>> 0); };
        for (const b of t.blobs) { mix(b.x); mix(b.y); mix(b.real ? 1 : 0); }
        const w = t.Engine.w;
        if (w) for (let i = 0; i < 3 * t.Engine.molecules; i++) { mix(w.holon_atom_x(i)); mix(w.holon_atom_y(i)); }
        const g = (id) => document.getElementById(id).textContent;
        return { digest: h, focus: g('h-focus'), real: g('h-real'), blobs: g('h-blobs'), engine: g('h-engine'), molecules: t.Engine.molecules };
    })()""",
    "per_frame_js": "window.__toy && window.__toy.stage ? window.__toy.stage.lastFrame : null",
    "hide_css": "#banner, #hud, #legend { visibility: hidden !important; }",
    "shot": "#stage",
}

res = harness.compare("play", check, repo, commit, out, frames=FRAMES)
A, B = res["after"], res["before"]
wall = subprocess.run(["node", os.path.join(repo, "docs", "play", "wall.mjs")], capture_output=True, text=True)
stage_ctx = [c for c in A["contexts"] if c["canvas"].startswith("stage")]
frames_ok = [f for f in A["per_frame"] if f["gl"].get("stage", 0) > 0 and f["page"] and f["gl"].get("stage", 0) == f["page"]["drawCalls"]]
conds = [
    ("C1 no page or console error (after)", not A["errors"] and not A["console_errors"], A["errors"] + A["console_errors"]),
    ("C2 #stage has a WebGL context and never a 2d one", bool(stage_ctx) and all(c["type"] in ("webgl2", "webgl") for c in stage_ctx), stage_ctx),
    ("C3 no Canvas-2D drawing call on any canvas", not A["draw2d_total"], A["draw2d_total"]),
    (f"C4 every frame's stage draws are draw.js's ({len(frames_ok)}/{FRAMES})", len(frames_ok) == FRAMES, A["per_frame"][:3]),
    ("C5 the same state before and after", A["state"] == B["state"], {"before": B["state"], "after": A["state"]}),
    ("C6 engine live, the ring holds real molecules", A["state"]["engine"].startswith("live, dt") and A["state"]["molecules"] > 0, A["state"]),
    ("C7 before/after pair and diff saved", "error" not in res["diff"], res["diff"]),
    ("C8 docs/play/wall.mjs passes", wall.returncode == 0, wall.stdout[-400:]),
]
report = {"commit_before": commit, "out": out, "gpu": A["gpu"], "diff_stage": res["diff"], "diff_page": res["diff_page"],
          "state_before": B["state"], "state_after": A["state"], "draws_per_frame_after": A["per_frame"][-1],
          "draws_per_frame_before": B["per_frame"][-1], "contexts_after": A["contexts"], "contexts_before": B["contexts"],
          "console_warnings_after": A["console_warnings"], "errors_before": B["errors"] + B["console_errors"]}
json.dump(report, open(os.path.join(out, "report.json"), "w"), indent=1)
print(json.dumps(report, indent=1)[:3000])
sys.exit(harness.verdict(conds))
