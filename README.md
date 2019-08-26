# Fontdue

Fontdue is a simple, `no_std`, pure Rust, truetype font parser and rasterizer. It aims to support all valid (unicode encoded) TrueType fonts correctly. It aims to make interacting with fonts as fast as possible. This includes getting layout information and rasterization.

A non-goal of this library is to be allocation free and have a fast/"zero cost" initial load. This libary _does_ make allocations and depends on the `alloc` crate. Fonts are fully parsed on creation and relevant information is stored in a more conveinent to accesss format. Unlike other font libraries, the font structures have no lifetime dependencies since it allocates its own space.

Ideally, font loading should be faster in the future, but making the loading process correct and readable was the initial priority.

## TrueType Tables
- `cmap` Character to glyph mapping (Unicode only)
  - Supports popular formats 0, 4, 6, 10, 12, 13
  - Planned support: formats 2, 8, 14
- `glyf` Glyph outlining
  - Planned support: Compound glyph matched points, compound glyph scaled offset
- `head` General font information
- `hhea` General horizontal layout
- `hmtx` Glyph horizontal layout
- `loca` Glyph outline offsets and lengths
- `maxp` Maximum values used for the font

Planned support for:
- `kern` Kerning pair layout
- `vhea` General vertical layout
- `vmtx` Glyph vertical layout


## Attribution

Inspired by how simple the wonderful `rusttype` crate made font parsing look.
Rasterizer from the `font-rs` crate.
