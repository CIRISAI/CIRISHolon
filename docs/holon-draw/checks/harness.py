"""The shared half of the pages' holon-draw checks (one check_<page>.py per ported page).

What it does, the same way for every page:
  * serves two docs/ trees on two local ports: BEFORE (the page as it was, a `git archive`
    of the commit named on the command line) and AFTER (this checkout's docs/);
  * opens each in real GPU Chrome (channel "chrome", ANGLE on EGL, the blocklist ignored);
  * before any page script runs, installs: a seeded Math.random, a virtual clock (every
    performance.now() call advances it by a fixed step, every pumped frame by 1000/60 ms),
    a requestAnimationFrame the check pumps by hand, and counters on every getContext() and
    on every WebGL draw call, per canvas;
  * pumps the same number of frames in both, so both show the SAME STATE (checked, not
    assumed: the page's own state digest must agree), and screenshots the stage in both;
  * writes before.png, after.png and diff.png and a pixel-difference summary.

The pass conditions are the page's (its check_<page>.py states them before the first run);
this file only measures.
"""
import json, os, subprocess, sys, time, socket, contextlib, shutil
import numpy as np
from PIL import Image
from playwright.sync_api import sync_playwright

INIT = r"""
(() => {
  const cfg = window.__CHECK_CFG__;
  // a seeded Math.random (mulberry32)
  let seed = cfg.seed >>> 0;
  Math.random = function () {
    seed |= 0; seed = (seed + 0x6D2B79F5) | 0;
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
  // the virtual clock
  let T = 1000;
  const realNow = performance.now.bind(performance);
  performance.now = () => (T += cfg.clockStep);
  // the hand-pumped frame
  let queue = [];
  window.__log = [];
  window.requestAnimationFrame = (cb) => { queue.push(cb); return queue.length; };
  window.cancelAnimationFrame = () => {};
  window.__pump = (n) => {
    for (let i = 0; i < n; i++) {
      const q = queue; queue = [];
      T += 1000 / 60;
      const now = T;
      window.__frameDraws = {};
      for (const cb of q) cb(now);
      // every pumped frame is logged: the draws per canvas, and what the page says it drew
      let pg = null;
      if (cfg.perFrame) { try { pg = (0, eval)(cfg.perFrame); } catch (e) { pg = { error: String(e) }; } }
      window.__log.push({ gl: Object.assign({}, window.__frameDraws), page: pg });
    }
    return queue.length;
  };
  window.__queued = () => queue.length;
  // getContext, per canvas
  window.__contexts = [];
  const name = (c) => (c instanceof HTMLCanvasElement ? (c.id || "(no id)") + (c.isConnected ? "" : " (detached)") : "OffscreenCanvas");
  for (const C of [HTMLCanvasElement, typeof OffscreenCanvas !== "undefined" ? OffscreenCanvas : null]) {
    if (!C) continue;
    const orig = C.prototype.getContext;
    C.prototype.getContext = function (type, ...rest) {
      const ctx = orig.call(this, type, ...rest);
      window.__contexts.push({ canvas: name(this), type, got: !!ctx });
      return ctx;
    };
  }
  // draws, per canvas, total and for the last pumped frame
  window.__draws = {}; window.__frameDraws = {};
  // draws made while no drawer's draw() is running (a page wraps draw() to set __inDrawJs)
  window.__drawsOutside = {}; window.__inDrawJs = 0;
  const count = (gl) => {
    const k = gl.canvas && gl.canvas.id ? gl.canvas.id : "(no id)";
    if (!window.__inDrawJs) window.__drawsOutside[k] = (window.__drawsOutside[k] || 0) + 1;
    window.__draws[k] = (window.__draws[k] || 0) + 1;
    window.__frameDraws[k] = (window.__frameDraws[k] || 0) + 1;
  };
  for (const P of [WebGL2RenderingContext.prototype, WebGLRenderingContext.prototype]) {
    for (const m of ["drawElementsInstanced", "drawArraysInstanced", "drawElements", "drawArrays"]) {
      if (!P[m]) continue;
      const o = P[m];
      P[m] = function (...a) { count(this); return o.apply(this, a); };
    }
  }
  // 2D drawing on a canvas in the document (fill/stroke/text/image), per canvas
  window.__draw2d = {};
  const P2 = CanvasRenderingContext2D.prototype;
  for (const m of ["fill", "stroke", "fillRect", "strokeRect", "fillText", "strokeText", "drawImage", "putImageData"]) {
    const o = P2[m];
    P2[m] = function (...a) { const k = this.canvas && this.canvas.id ? this.canvas.id : "(no id)"; window.__draw2d[k] = (window.__draw2d[k] || 0) + 1; return o.apply(this, a); };
  }
})();
"""


def free_port():
    with contextlib.closing(socket.socket()) as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def serve(root):
    port = free_port()
    p = subprocess.Popen([sys.executable, "-m", "http.server", str(port), "--bind", "127.0.0.1", "-d", root],
                         stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    for _ in range(100):
        try:
            with socket.create_connection(("127.0.0.1", port), 0.2):
                break
        except OSError:
            time.sleep(0.05)
    return p, port


def before_tree(repo, commit, out):
    """docs/ as it was at `commit`, extracted under out/ (once)."""
    dst = os.path.join(out, "before-" + commit)
    if not os.path.isdir(os.path.join(dst, "docs")):
        os.makedirs(dst, exist_ok=True)
        tar = subprocess.run(["git", "-C", repo, "archive", commit, "docs"], check=True, capture_output=True).stdout
        subprocess.run(["tar", "-x", "-C", dst], input=tar, check=True)
    return os.path.join(dst, "docs")


def diff_summary(a_path, b_path, out_path):
    a = np.asarray(Image.open(a_path).convert("RGB")).astype(np.int16)
    b = np.asarray(Image.open(b_path).convert("RGB")).astype(np.int16)
    if a.shape != b.shape:
        return {"error": f"sizes differ {a.shape} vs {b.shape}"}
    d = np.abs(a - b).max(axis=2)
    n = d.size
    Image.fromarray(np.clip(d * 4, 0, 255).astype(np.uint8)).save(out_path)
    return {
        "pixels": int(n),
        "identical_frac": round(float((d == 0).sum()) / n, 5),
        "differ_gt8_frac": round(float((d > 8).sum()) / n, 5),
        "differ_gt32_frac": round(float((d > 32).sum()) / n, 5),
        "differ_gt96_frac": round(float((d > 96).sum()) / n, 5),
        "mean_abs": round(float(d.mean()), 3),
        "max": int(d.max()),
    }


def run_page(url, cfg, frames, ready_js, state_js, hide_css, shot_selector, shot_path, viewport, extra=None, per_frame_js=None):
    """Open url, pump `frames` frames once ready, screenshot `shot_selector` with `hide_css` applied."""
    with sync_playwright() as p:
        browser = p.chromium.launch(channel="chrome", headless=True,
                                    args=["--use-angle=gl-egl", "--ignore-gpu-blocklist"])
        page = browser.new_page(viewport=viewport, device_scale_factor=1)
        errors, consoles = [], []
        page.on("pageerror", lambda e: errors.append(str(e)))
        page.on("console", lambda m: consoles.append((m.type, m.text + " @ " + str((m.location or {}).get("url", "")))))
        cfg = dict(cfg, perFrame=per_frame_js)
        page.add_init_script("window.__CHECK_CFG__ = " + json.dumps(cfg) + ";\n" + INIT)
        page.goto(url, wait_until="load")
        t0 = time.time()
        while not page.evaluate(ready_js):
            if time.time() - t0 > 60:
                raise RuntimeError("the page never became ready: " + ready_js)
            # boot may await a frame: let one through if it is waiting
            if page.evaluate("window.__queued()") and cfg.get("pumpWhileBooting"):
                page.evaluate("window.__pump(1)")
            time.sleep(0.05)
        page.evaluate(f"window.__pump({int(frames)})")
        if extra:
            extra(page)
        state = page.evaluate(state_js)
        gpu = page.evaluate("""(() => { const c = document.createElement('canvas'); const g = c.getContext('webgl2');
          const e = g && g.getExtension('WEBGL_debug_renderer_info'); return e ? g.getParameter(e.UNMASKED_RENDERER_WEBGL) : 'unknown'; })()""")
        page.wait_for_timeout(100)
        page.screenshot(path=shot_path.replace(".png", "-page.png"), full_page=False)  # the page as a reader sees it
        if hide_css:
            page.add_style_tag(content=hide_css)
        page.evaluate("window.__pump(1)")  # one more frame after the style change, same for both
        page.wait_for_timeout(100)
        page.locator(shot_selector).screenshot(path=shot_path)
        result = {
            "errors": errors,
            "console_errors": [t for (k, t) in consoles if k == "error"],
            "console_warnings": [t for (k, t) in consoles if k == "warning"],
            "contexts": page.evaluate("window.__contexts"),
            "draws_total": page.evaluate("window.__draws"),
            "draw2d_total": page.evaluate("window.__draw2d"),
            "draws_outside_drawjs": page.evaluate("window.__drawsOutside"),
            "per_frame": page.evaluate("window.__log"),
            "state": state,
            "gpu": gpu,
        }
        browser.close()
        return result


def compare(page_dir, check, repo, before_commit, out_dir, frames=60, **kw):
    """Run the page BEFORE and AFTER; return both results and the diff."""
    os.makedirs(out_dir, exist_ok=True)
    after_root = os.path.join(repo, "docs")
    before_root = before_tree(repo, before_commit, out_dir)
    res = {}
    for label, root in (("before", before_root), ("after", after_root)):
        srv, port = serve(root)
        try:
            res[label] = run_page(f"http://127.0.0.1:{port}/{page_dir}/{kw.get('query', '')}", check["cfg"], frames,
                                  check["ready"], check["state"], check.get("hide_css", ""), check["shot"],
                                  os.path.join(out_dir, f"{label}.png"), check.get("viewport", {"width": 1280, "height": 800}),
                                  check.get("extra"), check.get("per_frame_js"))
        finally:
            srv.terminate()
    res["diff"] = diff_summary(os.path.join(out_dir, "before.png"), os.path.join(out_dir, "after.png"), os.path.join(out_dir, "diff.png"))
    res["diff_page"] = diff_summary(os.path.join(out_dir, "before-page.png"), os.path.join(out_dir, "after-page.png"), os.path.join(out_dir, "diff-page.png"))
    return res


def verdict(conditions):
    """conditions: list of (name, bool, detail). Prints them; returns the exit code."""
    bad = 0
    for name, ok, detail in conditions:
        print(("  PASS " if ok else "  FAIL ") + name + ("" if ok else f"  -- {detail}"))
        bad += 0 if ok else 1
    print("\nall pass conditions hold" if bad == 0 else f"\n{bad} pass condition(s) FAILED")
    return 0 if bad == 0 else 1
