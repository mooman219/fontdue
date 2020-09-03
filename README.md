# Fontdue

[![Documentation](https://travis-ci.org/mooman219/fontdue.svg?branch=master)](https://travis-ci.org/mooman219/fontdue)
[![Documentation](https://docs.rs/fontdue/badge.svg)](https://docs.rs/fontdue)
[![Crates.io](https://img.shields.io/crates/v/fontdue.svg)](https://crates.io/crates/fontdue)
[![License](https://img.shields.io/crates/l/fontdue.svg)](https://github.com/mooman219/fontdue/blob/master/LICENSE)

Fontdue is a simple, `no_std` (does not use the standard library for portability), pure Rust, TrueType (`.ttf/.ttc`) & OpenType (`.otf`) font rasterizer and layout tool. It strives to make interacting with fonts as fast as possible, and currently has the fastest rasterizer.

## Roadmap

**Version 1.0:** `fontdue` is designed to be a replacement for `rusttype`, `ab_glyph`, and parts of `glyph_brush`. This is a class of font libraries that don't tackle shaping.

**Version 2.0:** Shaping - the complex layout of text such as Arabic and Devanagari - will be added. There are two potential pure Rust libraries (allsorts or rustybuzz) that are candidates for providing a shaping backend to Fontdue, but are relatively immature right now.

A **non-goal** of this library is to be allocation free and have a fast, "zero cost" initial load. This library _does_ make allocations and depends on the `alloc` crate. Fonts are fully parsed on creation and relevant information is stored in a more convenient to access format. Unlike other font libraries, the font structures have no lifetime dependencies since it allocates its own space.

## Example

### Rasterization
```rust
// Read the font data.
let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];
// Parse it into the font type.
let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
// Rasterize and get the layout metrics for the letter 'g' at 17px.
let (metrics, bitmap) = font.rasterize('g', 17.0);
```

### Layout
```rust
// Read the font data.
let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];
// Parse it into the font type.
let roboto_regular = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
// Create a layout context. This stores transient state needed to layout text.
// Laying out text needs some heap allocations; reusing this context reduces the need to reallocate space.
let mut layout = Layout::new();
// The vector where the glyphs positional information will be written to. This vec is cleared before it's written to.
let mut output = Vec::new();
// Various settings for laying out the text, such as alignment and wrapping settings.
let settings = LayoutSettings {
    ..LayoutSettings::default()
};
// The list of fonts that will be used during layout.
let fonts = &[roboto_regular];
// The text that will be laid out, its size, and the index of the font in the font list to use for that section of text.
let styles = &[
    &TextStyle::new("Hello ", 35.0, 0),
    &TextStyle::new("world!", 40.0, 0),
];
// Calculate the layout.
layout.layout_horizontal(fonts, styles, &settings, &mut output);
```

## Performance

### Rasterization

This benchmark measures the time it takes to generate the glyph metrics and bitmap for the letter 'g' over a range of sizes. Lower is better. This is using the idiomatic APIs for `rusttype` [(link)](https://gitlab.redox-os.org/redox-os/rusttype), `ab_glyph` [(link)](https://github.com/alexheretic/ab-glyph), and `fontdue`.

![Rasterize benchmarks](/images/rasterize.png)

Older versions of `rusttype` use a naive rasterizer that's roughly 10x slower than `fontdue`. The newer `rusttype` shares the same rasterizer as `ab_glyph` and `glyph_brush` [(link)](https://github.com/alexheretic/glyph-brush/tree/master/glyph-brush).

### Layout

This benchmark measures the time it takes to layout latin characters of sample text with wrapping on word boundaries. This is using the idiomatic APIs for both `glyph_brush_layout` [(link)](https://github.com/alexheretic/glyph-brush/tree/master/layout) and `fontdue`.

![Layout benchmarks](/images/layout.png)

## Notices

### Maintenance

Please bear with me on new features or quirks that you find. I will definitely get to issues you open (also thank you for opening them), but I don't have as much time as I would like to work on fontdue so please be patient, this is a mostly solo project <3.

### TrueType & OpenType Table Support

Fontdue depends on `ttf-parser` ([link](https://github.com/RazrFalcon/ttf-parser)) for parsing fonts, which supports a wide range of TrueType and OpenType features.

### Attribution

`Fontdue` started as a slightly more production ready wrapper around `font-rs` [(link)](https://github.com/raphlinus/font-rs) because of how fast it made rasterization look, and how simple the wonderful `rusttype` [(link)](https://gitlab.redox-os.org/redox-os/rusttype) crate made font parsing look. Since then, I've rewritten fontdue from the ground up, but I feel like it still deservers some attribution.