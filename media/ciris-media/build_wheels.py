#!/usr/bin/env python3
"""Build the four ciris-media wheels: each is one platform's two native jars, byte for byte.

    python3 build_wheels.py --out dist                 fetch from Maven Central, check, build
    python3 build_wheels.py --out dist --from-dir D    take the jars from D instead of the network
    python3 build_wheels.py --out dist --record        also print each jar's sha256, for pins.json
    python3 build_wheels.py --out dist --require-pins  fail if any sha256 pin is null (publishing)

A jar is used only if its size and sha1 are Maven Central's as recorded in pins.json, and its
sha256 equals the pin where one is set. Wheels are written with fixed timestamps and a fixed
member order, so the same inputs give the same wheel bytes. Standard library only.
"""
import argparse, base64, hashlib, json, pathlib, sys, urllib.request, zipfile

HERE = pathlib.Path(__file__).resolve().parent
FIXED = (2026, 1, 1, 0, 0, 0)


def fetch(name, pin, base, from_dir):
    if from_dir:
        data = (pathlib.Path(from_dir) / name).read_bytes()
    else:
        with urllib.request.urlopen(f"{base}/{pin['path']}/{name}", timeout=300) as r:
            data = r.read()
    if len(data) != pin["bytes"]:
        sys.exit(f"{name}: {len(data)} bytes, pins.json says {pin['bytes']}")
    if hashlib.sha1(data).hexdigest() != pin["sha1"]:
        sys.exit(f"{name}: sha1 {hashlib.sha1(data).hexdigest()}, pins.json says {pin['sha1']}")
    got = hashlib.sha256(data).hexdigest()
    if pin["sha256"] is not None and got != pin["sha256"]:
        sys.exit(f"{name}: sha256 {got}, pins.json says {pin['sha256']}")
    return data, got


def wheel(out, version, tag, files):
    name = f"ciris_media-{version}-py3-none-{tag}.whl"
    info = f"ciris_media-{version}.dist-info"
    meta = (f"Metadata-Version: 2.1\nName: ciris-media\nVersion: {version}\n"
            "Summary: The FFmpeg and JavaCPP native jars the CIRIS client's video calls load, unmodified from Maven Central\n"
            "Home-page: https://github.com/CIRISAI/CIRISHolon\n"
            "License: AGPL-3.0-only AND LGPL-3.0-or-later AND (Apache-2.0 OR GPL-2.0-or-later WITH Classpath-exception-2.0)\n"
            "Requires-Python: >=3.8\nDescription-Content-Type: text/markdown\n\n" + (HERE / "NOTICE.md").read_text(encoding="utf-8"))
    files = dict(files)
    files[f"{info}/METADATA"] = meta.encode()
    files[f"{info}/WHEEL"] = f"Wheel-Version: 1.0\nGenerator: ciris-media build_wheels.py\nRoot-Is-Purelib: false\nTag: py3-none-{tag}\n".encode()
    files[f"{info}/licenses/NOTICE.md"] = (HERE / "NOTICE.md").read_bytes()
    rec = "".join(f"{p},sha256={base64.urlsafe_b64encode(hashlib.sha256(b).digest()).rstrip(b'=').decode()},{len(b)}\n" for p, b in sorted(files.items()))
    files[f"{info}/RECORD"] = (rec + f"{info}/RECORD,,\n").encode()
    with zipfile.ZipFile(out / name, "w") as z:
        for p in sorted(files):
            zi = zipfile.ZipInfo(p, FIXED)
            zi.external_attr = 0o644 << 16
            # the jars are already compressed: store them, so the wheel's bytes do not depend on a zlib version
            zi.compress_type = zipfile.ZIP_STORED
            z.writestr(zi, files[p])
    return out / name


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    ap.add_argument("--from-dir")
    ap.add_argument("--pins", default=str(HERE / "pins.json"))
    ap.add_argument("--record", action="store_true")
    ap.add_argument("--require-pins", action="store_true")
    a = ap.parse_args()
    pins = json.loads(pathlib.Path(a.pins).read_text(encoding="utf-8"))
    if a.require_pins and any(j["sha256"] is None for j in pins["jars"].values()):
        sys.exit("a sha256 pin is null: nothing is built for publishing until every jar is pinned")
    out = pathlib.Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    init = (HERE / "src" / "ciris_media" / "__init__.py").read_bytes()
    if f'__version__ = "{pins["version"]}"'.encode() not in init:
        sys.exit("the package's __version__ is not pins.json's version")
    sums = []
    for classifier, t in pins["targets"].items():
        files = {"ciris_media/__init__.py": init}
        for name, pin in sorted(pins["jars"].items()):
            if name.endswith(f"-{classifier}.jar"):
                data, got = fetch(name, pin, pins["base"], a.from_dir)
                files[f"ciris_media/jars/{name}"] = data
                if a.record:
                    print(f"jar {name} sha256 {got}")
        if len(files) != 3:
            sys.exit(f"{classifier}: expected two jars, found {len(files) - 1}")
        w = wheel(out, pins["version"], t["wheel_tag"], files)
        sums.append(f"{hashlib.sha256(w.read_bytes()).hexdigest()}  {w.name}")
        print(f"wheel {w.name} {w.stat().st_size} bytes")
    (out / "SHA256SUMS").write_text("\n".join(sorted(sums, key=lambda s: s.split()[1])) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
