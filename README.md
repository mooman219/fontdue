# Fontdue

[![Documentation](https://travis-ci.org/mooman219/fontdue.svg?branch=master)](https://travis-ci.org/mooman219/fontdue)
[![Documentation](https://docs.rs/fontdue/badge.svg)](https://docs.rs/fontdue)
[![Crates.io](https://img.shields.io/crates/v/fontdue.svg)](https://crates.io/crates/fontdue)
[![License](https://img.shields.io/crates/l/fontdue.svg)](https://github.com/mooman219/fontdue/blob/master/LICENSE)

Fontdue is a simple, `no_std`, pure Rust, TrueType & OpenType font rasterizer and layout tool. It strives to make interacting with fonts as fast as possible.

A non-goal of this library is to be allocation free and have a fast, "zero cost" initial load. This library _does_ make allocations and depends on the `alloc` crate. Fonts are fully parsed on creation and relevant information is stored in a more convenient to access format. Unlike other font libraries, the font structures have no lifetime dependencies since it allocates its own space.

## Important Notices

### Maintenance

Please bear with me on new features or quirks that you find. I will definitely get to issues you open (also thank you for opening them), but I don't have as much time as I would like to work on fontdue so please be paitent, this is a mostly solo project <3.

### Reusing Fontdue code

Please don't reuse `fontdue`'s raster code directly in your project. `fontdue` uses **unsafe** code in the rasterizer, and the rasterizer itself is **very not safe** to use on its own with un-sanitized input.

## TrueType & OpenType Table Support

Fontdue now depends on `ttf-parser` ([link](https://github.com/RazrFalcon/ttf-parser)). There is a lot of work involved in parsing font tables, and I only had the resolve to write a parser for TypeType. The wonderful developer on `ttf-parser` has done a lot of great work and supports some OpenType tables, so I opted to use that library.

## Example

### Glyph Rasterization
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

### Metrics + Rasterize

This benchmark measures the time it takes to generate the glyph metrics and bitmap for the letter 'g' over a range of sizes. This is using the idiomatic APIs for both `rusttype` [(link)](https://gitlab.redox-os.org/redox-os/rusttype) and `fontdue`, and represents realworld performance.

This benchmarks is also representative of `glyph_brush` [(link)](https://github.com/alexheretic/glyph-brush/tree/master/glyph-brush) and `ab_glyph` [(link)](https://github.com/alexheretic/ab-glyph), as `rusttype`, `glyph_brush`, and `ab_glyph` use the same rasterizer from `font-rs` [(link)](https://github.com/raphlinus/font-rs). Older versions of `rusttype` use a naive rasterizer that's roughly 10x slower than `fontdue`.

```
rusttype 0.9.2 metrics + rasterize 'g'/20 time: [2.6128 us 2.6150 us 2.6171 us]
rusttype 0.9.2 metrics + rasterize 'g'/40 time: [4.3557 us 4.3595 us 4.3636 us]
rusttype 0.9.2 metrics + rasterize 'g'/60 time: [6.7007 us 6.7073 us 6.7140 us]
rusttype 0.9.2 metrics + rasterize 'g'/80 time: [10.021 us 10.031 us 10.040 us]

fontdue latest metrics + rasterize 'g'/20 time: [0.8599 us 0.8608 us 0.8619 us]
fontdue latest metrics + rasterize 'g'/40 time: [1.3236 us 1.3254 us 1.3277 us]
fontdue latest metrics + rasterize 'g'/60 time: [1.9306 us 1.9342 us 1.9385 us]
fontdue latest metrics + rasterize 'g'/80 time: [2.6937 us 2.6988 us 2.7051 us]
```

### Rich Layout

This benchmark measures the time it takes to layout 300 characters of sample text with wrapping on word boundaries. This is using the idiomatic APIs for both `glyph_brush_layout` [(link)](https://github.com/alexheretic/glyph-brush/tree/master/layout) and `fontdue`, and represents realword performance.

```
glyph_brush_layout 0.2.0 layout time: [40.051 us 40.133 us 40.224 us]

fontdue latest layout time:           [6.6665 us 6.6718 us 6.6777 us]
```

## Attribution

`Fontdue` started as a slightly more production ready wrapper around `font-rs` [(link)](https://github.com/raphlinus/font-rs) because of how fast it made rasterization look, and how simple the wonderful `rusttype` [(link)](https://gitlab.redox-os.org/redox-os/rusttype) crate made font parsing look. Since then, I've done a few rewrites on the raster and it no longer shares any code or methodology to `font-rs`, but I feel like it still deservers some attribution. Instead of attempting to find the converage of a pixel, `fontdue` performs pseudo ray tracing collision detection on the geometry of the glyph with the pixel grid and estimates the shading.