"""ciris-media: the FFmpeg and JavaCPP native jars the CIRIS client's video calls load, and nothing else.

This package holds no code that decodes anything. It carries, byte for byte, the native jars
published on Maven Central by the bytedeco project for one platform, so that the client's own
wheel does not have to. The client's jar finds them beside it in site-packages, or asks here.
"""
import pathlib

__version__ = "7.1.0"
FFMPEG = "7.1-1.5.11"
JAVACPP = "1.5.11"


def jar_dir() -> pathlib.Path:
    """The directory holding the native jars."""
    return pathlib.Path(__file__).resolve().parent / "jars"


def jars() -> list:
    """The native jars of this platform's wheel, sorted by name."""
    return sorted(jar_dir().glob("*.jar"))
