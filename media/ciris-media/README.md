# ciris-media

One Python wheel per desktop platform, holding that platform's FFmpeg and JavaCPP native jars
exactly as the bytedeco project publishes them on Maven Central. Nothing is compiled or changed.

**Why.** The CIRIS client's desktop wheels carried these jars (19 to 24 MiB each) and sat
within 0.18 to 3.6 MiB of PyPI's 100 MiB file limit, with the project near PyPI's 10 GiB
total. With the jars here, under their own version that changes only when FFmpeg does, each
client wheel is about 20 MiB smaller and each client release adds about 90 MB less. Requirements
as the client session gave them: CIRISAI/CIRISHolon#9.

**What is in a wheel.** `ciris_media/jars/ffmpeg-7.1-1.5.11-<platform>.jar`,
`ciris_media/jars/javacpp-1.5.11-<platform>.jar`, a few lines of Python that say where they
are (`ciris_media.jar_dir()`, `ciris_media.jars()`), and `NOTICE.md`.

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

**Status, 2026-11-23.** Not published. The sha256 pins are null until the first CI run has
printed them and they have been cross-checked; publishing refuses while any is null. The
PyPI project and its trusted publisher do not exist yet.

**Versions.** `7.1.0` is FFmpeg 7.1 with JavaCPP 1.5.11, as built by bytedeco. A later
version is meant to carry a cut-down build of the same libraries (the AV1 codecs and the
scaler only); that needs a build pipeline for four platforms and is not started.

**Terms.** `NOTICE.md`.
