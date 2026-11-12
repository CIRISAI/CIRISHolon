# APPEARANCE-0, the record before the parse

2026-11-12. Written before `appearance0.py run` reads a pixel. Freeze: `../APPEARANCE0_PREREG.md`.

- **The file.** `koppen_geiger_tif.zip`, 130,618,411 bytes, md5
  `7fc2f5a15d4f5fe0ce59c9a9b502aa09` (equal to the record's), sha256 `bb84453d…5562`,
  fetched from the address the repository's API gives for file 61012822. The record's
  licence field was read again at the fetch: "CC BY 4.0", version 2.
- **Read:** the archive's file list (73 entries; six period folders and `legend.txt`);
  the legend; the tags of `1991_2020/koppen_geiger_0p00833333.tif`. No pixel.
- **The raster.** Exactly one 1 km candidate under `1991_2020`. 43,200 by 21,600, 1 byte
  per pixel, upper-left corner at (−180, 90), pixel 1/120° (the tag reads
  0.008333333333333333), WGS84, no-data 0. So 15 by 15 pixels nest in each cell. Tiled,
  LZW. Requirement met.
- **The legend.** 30 entries, codes 1 to 30; code 1 Af, 4 BWh, 15 Cfb, 29 ET, 30 EF, as
  the freeze stated from memory. Requirement met. Its colours are written to the sidecar
  marked as the source's map colours.
- **The reader.** Pillow, with its pixel-count limit lifted (the image is 933 million
  pixels); the installed TIFF library has no LZW codec of its own here.
- **Citation as the legend asks:** Beck, H. E., T. R. McVicar, N. Vergopolan, A. Berg,
  N. J. Lutsko, A. Dufour, Z. Zeng, X. Jiang, A. I. J. M. van Dijk, and D. G. Miralles.
  High-resolution (1 km) Köppen-Geiger maps for 1901–2099 based on constrained CMIP6
  projections, Scientific Data 10, 724 (2023).
