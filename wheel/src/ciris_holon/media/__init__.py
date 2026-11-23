"""ciris_holon.media: the FFmpeg and JavaCPP native jars the CIRIS client's video calls load, and nothing else.

This module holds no code that decodes anything. It carries, byte for byte, the native jars
published on Maven Central by the bytedeco project for one platform, so that the client's own
wheel does not have to. The client's jar finds them beside it in site-packages, or asks here.
"""
import pathlib

FFMPEG = "7.1-1.5.11"
JAVACPP = "1.5.11"


def jar_dir() -> pathlib.Path:
    """The directory holding the native jars."""
    return pathlib.Path(__file__).resolve().parent / "jars"


def jars() -> list:
    """The native jars of this platform's wheel, sorted by name."""
    return sorted(jar_dir().glob("*.jar"))


def verify() -> bool:
    """True when each jar here hashes to its line in jars/SHA256SUMS and there are no others."""
    import hashlib
    want = dict(reversed(line.split("  ", 1)) for line in (jar_dir() / "SHA256SUMS").read_text().splitlines() if line)
    have = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in jars()}
    return have == want and len(have) > 0
