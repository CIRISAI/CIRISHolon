#!/usr/bin/env python3
"""The builder against stand-in jars (no network): the wheels hold exactly the bytes given, install-shaped,
identical on a second build, and a wrong byte, a wrong size or a missing pin is refused. Run: python3 test_build.py"""
import hashlib, json, pathlib, subprocess, sys, tempfile, zipfile

HERE = pathlib.Path(__file__).resolve().parent
pins = json.loads((HERE / "pins.json").read_text())
with tempfile.TemporaryDirectory() as d:
    d = pathlib.Path(d); src = d / "jars"; src.mkdir()
    fake = json.loads(json.dumps(pins))
    for name, pin in fake["jars"].items():
        data = (name * 7).encode()
        (src / name).write_bytes(data)
        pin.update(bytes=len(data), sha1=hashlib.sha1(data).hexdigest(), sha256=hashlib.sha256(data).hexdigest())
    (d / "pins.json").write_text(json.dumps(fake))
    run = lambda out, *extra: subprocess.run([sys.executable, str(HERE / "build_wheels.py"), "--out", str(d / out), "--from-dir", str(src), "--pins", str(d / "pins.json"), *extra], capture_output=True, text=True)
    r = run("a", "--require-pins"); assert r.returncode == 0, r.stderr
    assert run("b").returncode == 0
    wheels = sorted((d / "a").glob("*.whl")); assert len(wheels) == 4, wheels
    for w in wheels:
        assert w.read_bytes() == (d / "b" / w.name).read_bytes(), f"{w.name} differs between two builds"
        with zipfile.ZipFile(w) as z:
            names = z.namelist(); jars = [n for n in names if n.endswith(".jar")]
            assert len(jars) == 2 and "ciris_media/__init__.py" in names, names
            for j in jars:
                assert z.read(j) == (src / pathlib.Path(j).name).read_bytes()
            sums = z.read("ciris_media/jars/SHA256SUMS").decode().split("\n")
            assert sorted(sums[:-1]) == sorted(f"{hashlib.sha256(z.read(j)).hexdigest()}  {pathlib.Path(j).name}" for j in jars), sums
            rec = z.read([n for n in names if n.endswith("RECORD")][0]).decode()
            assert all(n in rec for n in names)
    tags = {w.name.split("-py3-none-")[1][:-4] for w in wheels}
    assert tags == {t["wheel_tag"] for t in pins["targets"].values()}, tags
    # refusals
    one = next(iter(fake["jars"]))
    (src / one).write_bytes((src / one).read_bytes()[:-1] + b"!")
    assert run("c").returncode != 0, "a changed byte was accepted"
    (src / one).write_bytes(b"short")
    assert run("c").returncode != 0, "a wrong size was accepted"
    (d / "nopin.json").write_text(json.dumps(dict(fake, jars={k: dict(v, sha256=None) for k, v in fake["jars"].items()})))
    r = subprocess.run([sys.executable, str(HERE / "build_wheels.py"), "--out", str(d / "e"), "--from-dir", str(src), "--pins", str(d / "nopin.json"), "--require-pins"], capture_output=True, text=True)
    assert r.returncode != 0 and "pin is null" in r.stderr, "--require-pins let a null pin through"
print("ok: four wheels, two jars each, byte-identical on rebuild; a changed byte, a wrong size and a null pin are refused")
