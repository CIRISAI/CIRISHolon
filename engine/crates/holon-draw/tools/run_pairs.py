#!/usr/bin/env python3
"""Interleaved BEFORE/AFTER runs using compare_pages.one_run; writes compare.json."""
import argparse, json, os, statistics, sys, time
sys.path.insert(0, "/home/emoore/CIRISHolon/engine/crates/holon-draw/tools")
import compare_pages as cp
from playwright.sync_api import sync_playwright

# Outputs and the BEFORE root live in a working directory given by HOLON_COMPARE_DIR (the
# BEFORE root holds docs/atoms3d/index.html plus a Bevy webgl2 build, which is not committed).
S = os.environ.get("HOLON_COMPARE_DIR") or os.path.dirname(os.path.abspath(__file__))
PAGES = {
    "before_head": dict(root=os.path.join(S, "before_head_root"), page="atoms3d/index.html",
        ready='!!document.getElementById("boot") && document.getElementById("boot").hidden',
        fail='!!document.querySelector("#boot-note.error")'),
    "after": dict(root=cp.DEFAULT_ROOT, page="atoms-draw/index.html",
        ready='document.body.dataset.ready === "1"',
        fail='(document.body.dataset.engine || "").startsWith("failed")'),
    "after_dpr1": dict(root=cp.DEFAULT_ROOT, page="atoms-draw/index.html?dpr=1",
        ready='document.body.dataset.ready === "1"',
        fail='(document.body.dataset.engine || "").startsWith("failed")'),
}
PLAN = [("desktop", ["before_head", "after"], 5), ("phone", ["before_head", "after", "after_dpr1"], 3)]

servers = {}
def base(root):
    if root not in servers:
        servers[root] = cp.serve(root)
    return f"http://127.0.0.1:{servers[root][0].server_address[1]}/"

results = {}
try:
    with sync_playwright() as pw:
        for mode, labels, n in PLAN:
            for i in range(1, n + 1):
                for label in labels:
                    p = PAGES[label]
                    a = argparse.Namespace(page=p["page"], ready=p["ready"], fail=p["fail"], pixels="canvas",
                        pixel_fraction=0.002, timeout=240.0, window=10.0, out=S, label=label, root=p["root"])
                    r = cp.one_run(pw, a, mode, base(p["root"]), i)
                    r["label"] = label; r["order"] = time.strftime("%H:%M:%S")
                    results.setdefault(f"{label}|{mode}", []).append(r)
                    print(mode, i, label, r["error"] or (r["ready_js_ms"], r["ready_pixels_ms"],
                          r["frames_harness_clock"], r["canvases"], r["host_load1_before"]), flush=True)
finally:
    for httpd, th in servers.values():
        httpd.shutdown(); httpd.server_close(); th.join(timeout=5)

out = dict(caveat=cp.CAVEAT, chromium_args=cp.CHROMIUM_ARGS, modes=cp.MODES, pages=PAGES,
           interleaved="within each mode, one run of each page in turn, a new browser process per run",
           window_s=10.0, pixels_selector="canvas", cells={})
for key, runs in results.items():
    good = [r for r in runs if not r["error"]]
    summ = {}
    for k, get in cp.SUMMARY_KEYS:
        v = [get(r) for r in good if get(r) is not None]
        summ[k] = dict(median=round(statistics.median(v), 2), min=min(v), max=max(v)) if v else None
    out["cells"][key] = dict(runs_ok=len(good), runs=len(runs), summary=summ,
        canvases=good[0]["canvases"] if good else None, engine=good[0]["engine"] if good else None,
        graphics_path=good[0]["graphics_path"] if good else None,
        webgl_renderer=good[0]["webgl_renderer"] if good else None,
        host_load1=[r["host_load1_before"] for r in runs],
        console_errors=sum(len(r.get("console_errors", [])) + len(r.get("page_errors", [])) for r in runs),
        runs_detail=runs)
json.dump(out, open(os.path.join(S, "compare.json"), "w"), indent=1)
print("wrote compare.json")
