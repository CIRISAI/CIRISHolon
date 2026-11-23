# ciris-holon

What CIRISHolon publishes for other CIRIS components to run on. Version 0.1 holds one thing,
`ciris_holon.media`: one Python wheel per desktop platform, holding that platform's FFmpeg and JavaCPP native jars
exactly as the bytedeco project publishes them on Maven Central. Nothing is compiled or changed.

**Why.** The CIRIS client's desktop wheels carried these jars (19 to 24 MiB each) and sat
within 0.18 to 3.6 MiB of PyPI's 100 MiB file limit, with the project near PyPI's 10 GiB
total. With the jars here each client wheel is about 20 MiB smaller, and each client release adds
about 90 MB less so long as this package is not re-released with it.

**Why `ciris-holon` and not `ciris-media`.** Eric, 2026-11-23: "video is one form of holon we
draw". The draw engine's native build for the client (#9) is meant to land in this same
package. The cost: every release of this package carries the jars again (about 95 MB across
the four wheels) against this project's own PyPI total. Requirements
as the client session gave them: CIRISAI/CIRISHolon#9.

**What is in a wheel.** `ciris_holon/media/jars/ffmpeg-7.1-1.5.11-<platform>.jar`,
`ciris_holon/media/jars/javacpp-1.5.11-<platform>.jar`, `ciris_holon/media/jars/SHA256SUMS` (the two jars' digests, so a loader can check what it found),
a few lines of Python (`ciris_holon.media.jar_dir()`, `jars()`, `verify()`), and `NOTICE.md`.

The client's jar sits at `site-packages/ciris_client/_artifacts/`, so from there the jars are
at `../../ciris_holon/media/jars/<name>.jar`.

| platform | wheel tag |
|---|---|
| linux-x86_64 | `manylinux_2_17_x86_64` |
| macosx-x86_64 | `macosx_10_9_x86_64` |
| macosx-arm64 | `macosx_11_0_arm64` |
| windows-x86_64 | `win_amd64` |

**How it is checked.** `pins.json` lists each jar's size and sha1 as Maven Central states
them, and this repository's own sha256 pin. `build_wheels.py` refuses a jar that differs. The
wheels are written with fixed timestamps and order, so a rebuild gives the same bytes, and CI
builds twice and compares. `test_build.py` runs the builder on stand-in jars and plants a
changed byte, a wrong size and a missing pin.

**Status, 2026-11-23.** Not published. All eight jars are pinned by sha256. CI has fetched
them, built the four wheels twice to the same bytes, and installed one. The wheels are
19.5 to 27.5 MB. The GitHub environment `pypi` exists; the PyPI project and its trusted
publisher do not, and only the account owner can make them:

| PyPI "pending publisher" field | value |
|---|---|
| PyPI project name | `ciris-holon` |
| owner | `CIRISAI` |
| repository | `CIRISHolon` |
| workflow | `ciris-holon.yml` |
| environment | `pypi` |

Then: Actions, ciris-holon, Run workflow on main, with publish ticked.

**Versions.** `0.1.0` carries FFmpeg 7.1 with JavaCPP 1.5.11, as built by bytedeco
(`ciris_holon.media.FFMPEG`, `JAVACPP`). The minor version changes when a consumer must
change. A later version is meant to carry a cut-down build of the same libraries (the AV1 codecs and the
scaler only); that needs a build pipeline for four platforms and is not started.

**Terms.** `NOTICE.md`.
