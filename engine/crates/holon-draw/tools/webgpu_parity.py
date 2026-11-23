#!/usr/bin/env python3
"""Draw test.html (WebGL) and test-webgpu.html (WebGPU) the same way and compare their pixels.

    python3 crates/holon-draw/tools/webgpu_parity.py [--out DIR] [--width 960 --height 600]
                                                      [--modes default,map,noshadow,images] [--fps 4]

For each mode (default: the 3D scene with shadows; map: ?map=1; noshadow: ?shadow=0; images:
?images=1, a panel of host textures) and for
the scene alone (?overlay=0) and with the panels, it loads both pages in one headless Google
Chrome at the same size, device pixel ratio 1 and camera, waits for the page's own
`document.body.dataset.ready`, takes a screenshot of the canvas, and compares the two:

  mean_abs     mean absolute difference per channel (RGB, 0..255) over every pixel
  over_8       % of pixels whose largest channel difference is over 8
  over_32      % of pixels whose largest channel difference is over 32
  max          the largest channel difference anywhere

A diff picture (|a - b| x 4) is written beside the two screenshots. With --fps N each page is
also run with ?spin=1 for N seconds and its frames per second counted.

It also checks that the mesh builders copied into draw-webgpu.js are still the same source
text as draw.js's (they are copies; a drift would make the backends draw different meshes),
that each page reported ready with no error, that the WebGPU page's verdict is PASS, and that
both backends skip the same images for an empty slot (one, in the images mode with panels).
Exit status 0 when all of that holds; the pixel numbers are REPORTED, not gated (the two
backends rasterise on different paths, see the README).

WHAT THIS DOES NOT MEASURE. Chrome is headless with SwiftShader for both WebGL (ANGLE) and
WebGPU (Dawn on SwiftShader Vulkan): every GPU operation runs on the CPU. Pixel agreement
here says the WGSL ports compute what the GLSL does on one software rasteriser; it says
nothing about a hardware GPU. Frames per second here are SwiftShader's and say nothing about
any device's frame time; they compare the two pages under the same conditions only.

No network beyond 127.0.0.1. Needs Python Playwright and /usr/bin/google-chrome.
"""

import argparse
import base64
import functools
import http.server
import json
import os
import re
import sys
import threading

from playwright.sync_api import sync_playwright

HERE = os.path.dirname(os.path.abspath(__file__))
WEB = os.path.abspath(os.path.join(HERE, "..", "web"))
CHROME = "/usr/bin/google-chrome"
# What runs WebGPU headless on this Linux box: Dawn's SwiftShader Vulkan adapter. Without
# --use-vulkan=swiftshader requestAdapter() returned null in most launches here; with
# --disable-vulkan-surface the adapter works but the canvas is not composited, so a screenshot
# is blank. These flags gave an adapter and a composited canvas in every launch tried.
CHROME_ARGS = [
    "--enable-unsafe-webgpu",
    "--enable-features=Vulkan",
    "--use-vulkan=swiftshader",
    "--use-webgpu-adapter=swiftshader",
    "--use-angle=swiftshader",
    "--enable-unsafe-swiftshader",
]
MODES = {"default": "", "map": "map=1", "noshadow": "shadow=0", "images": "images=1"}
MESH_FUNCS = ["sphereMesh", "tubeMesh", "capsuleMesh", "arcCapsuleMesh", "arcRibbonMesh",
              "ribbonMesh", "boxMesh", "quadMesh", "discMesh"]

COMPARE_JS = """
async ([a, b]) => {
  const load = async (b64) => {
    const blob = await (await fetch('data:image/png;base64,' + b64)).blob();
    const bmp = await createImageBitmap(blob, { colorSpaceConversion: 'none', premultiplyAlpha: 'none' });
    const c = new OffscreenCanvas(bmp.width, bmp.height);
    const x = c.getContext('2d');
    x.drawImage(bmp, 0, 0);
    return x.getImageData(0, 0, bmp.width, bmp.height);
  };
  const A = await load(a), B = await load(b);
  if (A.width !== B.width || A.height !== B.height) return { error: 'sizes differ' };
  const n = A.width * A.height, da = A.data, db = B.data;
  const out = new ImageData(A.width, A.height);
  let sum = 0, o8 = 0, o32 = 0, mx = 0;
  for (let i = 0; i < n; i++) {
    let m = 0;
    for (let c = 0; c < 3; c++) {
      const d = Math.abs(da[4 * i + c] - db[4 * i + c]);
      sum += d; if (d > m) m = d;
      out.data[4 * i + c] = Math.min(255, d * 4);
    }
    out.data[4 * i + 3] = 255;
    if (m > 8) o8++;
    if (m > 32) o32++;
    if (m > mx) mx = m;
  }
  const c = new OffscreenCanvas(A.width, A.height);
  c.getContext('2d').putImageData(out, 0, 0);
  const png = await c.convertToBlob({ type: 'image/png' });
  const buf = new Uint8Array(await png.arrayBuffer());
  let s = '';
  for (let i = 0; i < buf.length; i += 0x8000) s += String.fromCharCode.apply(null, buf.subarray(i, i + 0x8000));
  return { width: A.width, height: A.height, mean_abs: +(sum / (3 * n)).toFixed(4),
           over_8: +(100 * o8 / n).toFixed(4), over_32: +(100 * o32 / n).toFixed(4), max: mx, diff_png: btoa(s) };
}
"""

FPS_JS = """
(ms) => new Promise((done) => {
  let n = 0; const t0 = performance.now();
  const f = () => { n++; const t = performance.now() - t0; if (t < ms) requestAnimationFrame(f); else done(n * 1000 / t); };
  requestAnimationFrame(f);
})
"""


class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *a):
        pass


def serve():
    srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Quiet, directory=WEB))
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    return srv


def mesh_drift():
    """Names of the mesh builders whose source differs between draw.js and draw-webgpu.js."""
    def funcs(path):
        src = open(path).read()
        out = {}
        for name in MESH_FUNCS:
            m = re.search(r"^function " + name + r"\(.*?^}\n", src, re.S | re.M)
            out[name] = m.group(0) if m else None
        return out
    a, b = funcs(os.path.join(WEB, "draw.js")), funcs(os.path.join(WEB, "draw-webgpu.js"))
    return [n for n in MESH_FUNCS if a[n] is None or a[n] != b[n]]


def load(browser, url, w, h, shot):
    page = browser.new_page(viewport={"width": w, "height": h}, device_scale_factor=1)
    errs = []
    page.on("pageerror", lambda e: errs.append(str(e)))
    page.goto(url)
    try:
        page.wait_for_function("document.body.dataset.ready === '1' || !!document.body.dataset.error", timeout=90000)
    except Exception as e:  # noqa: BLE001
        errs.append("not ready: " + str(e).splitlines()[0])
    info = page.evaluate("({ error: document.body.dataset.error || null, verdict: document.body.dataset.verdict || null,"
                         "  stats: document.body.dataset.stats ? JSON.parse(document.body.dataset.stats) : null })")
    png = page.locator("#c").screenshot(type="png") if shot else None
    return page, info, errs, png


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--out", default=os.path.join(os.environ.get("TMPDIR", "/tmp"), "holon-draw-webgpu-parity"))
    ap.add_argument("--width", type=int, default=960)
    ap.add_argument("--height", type=int, default=600)
    ap.add_argument("--modes", default="default,map,noshadow,images")
    ap.add_argument("--fps", type=float, default=0, help="seconds of ?spin=1 per page to count frames (0: skip)")
    args = ap.parse_args()
    os.makedirs(args.out, exist_ok=True)

    ok = True
    report = {"chrome_args": CHROME_ARGS, "size": [args.width, args.height], "results": [], "fps": []}
    drift = mesh_drift()
    report["mesh_drift"] = drift
    if drift:
        ok = False
        print("FAIL mesh builders differ between draw.js and draw-webgpu.js:", ", ".join(drift))
    else:
        print("ok   mesh builders in draw-webgpu.js are draw.js's, source for source")

    srv = serve()
    base = "http://127.0.0.1:%d/" % srv.server_address[1]
    with sync_playwright() as pw:
        browser = pw.chromium.launch(executable_path=CHROME, headless=True, args=CHROME_ARGS)
        report["chrome"] = browser.version
        print("chrome %s, %dx%d at dpr 1" % (browser.version, args.width, args.height))
        print("%-10s %-8s %10s %9s %9s %5s  %s" % ("mode", "overlay", "mean_abs", "over_8%", "over_32%", "max", "draw calls gl/gpu"))
        for mode in args.modes.split(","):
            for overlay in ("0", "1"):
                qs = "&".join(x for x in (MODES[mode], "overlay=0" if overlay == "0" else "") if x)
                pngs, infos = [], []
                for name in ("test.html", "test-webgpu.html"):
                    page, info, errs, png = load(browser, base + name + ("?" + qs if qs else ""), args.width, args.height, True)
                    page.close()
                    tag = "%s_%s_%s" % (mode, "scene" if overlay == "0" else "panels", "webgl" if name == "test.html" else "webgpu")
                    open(os.path.join(args.out, tag + ".png"), "wb").write(png)
                    if errs or info["error"] or (name == "test-webgpu.html" and info["verdict"] != "PASS"):
                        ok = False
                        print("FAIL %s?%s: %s" % (name, qs, errs or info["error"] or info["verdict"]))
                    pngs.append(base64.b64encode(png).decode())
                    infos.append(info)
                cmp_page = browser.new_page()
                cmp_page.goto(base + "test-scene.js")   # any same-origin page; the comparison is self-contained
                r = cmp_page.evaluate(COMPARE_JS, pngs)
                cmp_page.close()
                diff = r.pop("diff_png", None)
                if diff:
                    open(os.path.join(args.out, "%s_%s_diff.png" % (mode, "scene" if overlay == "0" else "panels")), "wb").write(base64.b64decode(diff))
                dc = [i["stats"]["drawCalls"] if i["stats"] else None for i in infos]
                inst = [i["stats"]["instances"] if i["stats"] else None for i in infos]
                if dc[0] != dc[1] or inst[0] != inst[1]:
                    ok = False
                    print("FAIL %s overlay=%s: draw calls %s, instances %s differ" % (mode, overlay, dc, inst))
                miss = [i["stats"].get("missingImages") if i["stats"] else None for i in infos]
                if miss[0] != miss[1] or (mode == "images" and overlay == "1" and miss[0] != 1):
                    ok = False
                    print("FAIL %s overlay=%s: images skipped for an empty slot %s (want 1 on both with the panels)" % (mode, overlay, miss))
                print("%-10s %-8s %10s %9s %9s %5s  %s/%s" % (mode, "on" if overlay == "1" else "off", r.get("mean_abs"), r.get("over_8"),
                                                          r.get("over_32"), r.get("max"), dc[0], dc[1]))
                report["results"].append(dict(mode=mode, overlay=overlay == "1", query=qs, **r,
                                              stats={"webgl": infos[0]["stats"], "webgpu": infos[1]["stats"]}))
        if args.fps > 0:
            for mode in args.modes.split(","):
                row = {"mode": mode}
                for name in ("test.html", "test-webgpu.html"):
                    qs = "&".join(x for x in (MODES[mode], "spin=1") if x)
                    page, info, errs, _ = load(browser, base + name + "?" + qs, args.width, args.height, False)
                    fps = page.evaluate(FPS_JS, int(args.fps * 1000)) if not errs and not info["error"] else None
                    page.close()
                    row["webgl" if name == "test.html" else "webgpu"] = round(fps, 2) if fps else None
                report["fps"].append(row)
                print("fps %-10s webgl %s  webgpu %s  (SwiftShader, ?spin=1, %g s)" % (mode, row["webgl"], row["webgpu"], args.fps))
        browser.close()
    srv.shutdown()
    with open(os.path.join(args.out, "parity.json"), "w") as f:
        json.dump(report, f, indent=1)
    print("pictures and parity.json in", args.out)
    print("PASS" if ok else "FAIL")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
