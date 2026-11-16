# Text in every script: what exists in Rust, and what is ours to build

*Research by a subagent of the CIRISHolon session, 2026-11-15, against crates.io and the
repositories on that date; the sizes are its own builds (wasm32-unknown-unknown, opt-level z,
LTO, panic=abort, no wasm-opt). Nothing here is built into the engine yet. The question was the
lead's: "Look for any rust libraries we can bring in to do the scripts in all our languages
clean and minimal, or else we may need to build our own holon compression standard."*

**Languages:** the client's 29: am ar bn de en es fa fr ha hi id it ja ko mr my pa pt ru sw
ta te th tr uk ur vi yo zh.

## The answer

An existing pure-Rust stack covers all 29 languages for bidirectional order, shaping, line
breaking and rasterising, and compiles for wasm32 and no_std (thumbv7em; not thumbv6m, which
lacks atomic compare-and-swap). An own FONT format is not justified: OpenType with WOFF2 and
subsetting is already the compact per-script form, and the shaping tables are irreplaceable
for Arabic and the Indic scripts.

| need | crate (version, last release) | licence |
|---|---|---|
| shaping | harfrust 0.16.0 (2026-11-12), HarfBuzz's own Rust port, successor to rustybuzz | MIT |
| font parsing, outlines | skrifa 0.50 + read-fonts 0.47 (fontations, 2026-11-10) | MIT/Apache |
| rasterising | zeno 0.3.3 | MIT/Apache |
| bidirectional order | unicode-bidi 0.3.18 | MIT/Apache |
| line breaking, Thai and Burmese dictionaries | icu_segmenter 2.4.0 (ICU4X, 2026-10-22) | Unicode-3.0 |
| subsetting (a build tool) | skera 0.10.0 (fontations) | MIT/Apache |

Measured size of that stack: 1,195 KB raw, 437 KB gzipped, without the Thai/Burmese line
breaking models; 736 KB gzipped with the LSTM models; 1,078 KB with dictionaries. The models can
be loaded at run time with those scripts' fonts instead. The engine's own core is 51 KB. 41
crates in the tree (10 build-time). Harfrust shaped sample text in the 15 non-trivial languages
with no missing glyph, Urdu Nastaliq with its cursive attachment active; there was no local
HarfBuzz to diff against, so conformance rests on upstream's reported results.

One-crate options fall short: cosmic-text does everything but breaks lines without
dictionaries, so Thai and Burmese do not wrap; parley lays out with dictionaries but has no
rasteriser.

## Fonts (Noto, OFL 1.1; gzip as an upper bound for WOFF2)

A base of Latin with Vietnamese, Yoruba, Hausa and Cyrillic is 109 KB. Each other non-CJK script
adds 19 to 220 KB (Thai 19, Tamil 36, Devanagari 96, Arabic 107, Ethiopic 144, Nastaliq 220);
all eleven together are about 1.0 MB. CJK dominates: Korean 290 KB for the common syllables,
Japanese and Chinese about 1.5 MB each at common-use coverage.

## What is ours to build

1. **Fallback on wasm and bare metal:** a table from script to font file and a lazy loader.
   Small.
2. **Distance fields for world-space labels:** no lean pure-Rust generator exists (fdsm brings
   nalgebra and image; msdfgen is C++). A single-channel distance transform over the
   rasteriser's coverage is short to write. Screen text stays coverage at device pixels.
3. **A build-time pack, if anything earns the name "holon format":** for each locale, the
   shaped glyph runs of its fixed strings and their atlas, made at build time by the stack above.
   A page then needs no shaper for static text, and CJK falls from megabytes to the glyphs the
   strings use. Dynamic text (names, messages, transcripts, documents) still needs the shaper
   and the fonts.

## The arrangement this suggests

On the web the browser is already a complete text provider at no download cost (the engine's
`web/text.js`), so the Rust stack is not needed there. Off the web (CEWPOS on bare metal, a
native client) the stack above is the provider behind the same `Text` trait, loaded as its own
module, with the build-time pack serving fixed strings.
