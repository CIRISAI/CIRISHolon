# What is in this package, and under what terms

This package redistributes, unmodified, native jars built and published by the bytedeco
project (https://github.com/bytedeco/javacpp-presets, tag 1.5.11) on Maven Central:

- `org.bytedeco:ffmpeg:7.1-1.5.11`, one platform classifier;
- `org.bytedeco:javacpp:1.5.11`, the same classifier.

`pins.json` in the source repository lists each file's size and digests.

**The wrapper code** in those jars (JavaCPP and its presets) is offered by its authors under
the Apache License 2.0, or the GNU GPL version 2 or later with the Classpath exception.

**The FFmpeg libraries** inside are the bytedeco default build, not its `-gpl` build. Its
build script (`ffmpeg/cppbuild.sh` at that tag) configures FFmpeg with `--enable-version3`
and without `--enable-gpl`, which makes the FFmpeg libraries LGPL version 3 or later. They
are shared libraries, and may be replaced by a build of your own. FFmpeg's source is at
https://ffmpeg.org; the exact sources and patches of this build are fetched by that script.

The same build statically includes other libraries under their own terms, among them
SVT-AV1, dav1d, libaom, libvpx, OpenSSL, LAME, Speex, opencore-amr, vo-amrwbenc and
OpenH264. This notice does not list every one; the build script does. OpenH264 is included
as compiled by bytedeco, not as the binary Cisco distributes.

**This package's own files** (`ciris_holon/`, the builder) are part of
CIRISHolon and under the GNU AGPL version 3.

Nothing here is legal advice. The digests say which bytes are carried; the terms above are
as their authors state them at the places named.
