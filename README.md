# Fontdue

[![Test](https://github.com/mooman219/fontdue/actions/workflows/build.yml/badge.svg)](https://github.com/mooman219/fontdue/actions)
[![Documentation](https://docs.rs/fontdue/badge.svg)](https://docs.rs/fontdue)
[![Crates.io](https://img.shields.io/crates/v/fontdue.svg)](https://crates.io/crates/fontdue)
[![License](https://img.shields.io/crates/l/fontdue.svg)](https://github.com/mooman219/fontdue/blob/master/LICENSE)

Fontdue is a simple, `no_std` (does not use the standard library for portability), pure Rust, TrueType (`.ttf/.ttc`) & OpenType (`.otf`) font rasterizer and layout tool. It strives to make interacting with fonts as fast as possible, and currently has the lowest end to end latency for a font rasterizer.

## Roadmap
`fontdue` is designed to be a replacement for `rusttype` [(link)](https://gitlab.redox-os.org/redox-os/rusttype), `ab_glyph` [(link)](https://github.com/alexheretic/ab-glyph), parts of `glyph_brush` [(link)](https://github.com/alexheretic/glyph-brush/tree/master/glyph-brush), and `glyph_brush_layout` [(link)](https://github.com/alexheretic/glyph-brush/tree/master/layout). This is a class of font libraries that don't tackle shaping.

It's unlikely I'll have time to integrate with a shaping library. If you need a more complete font engine you should check out the wonderful [Cosmic Text](https://github.com/pop-os/cosmic-text) project, which is a very complete pure rust text library.

A **non-goal** of this library is to be allocation free and have a fast, "zero cost" initial load. This library _does_ make allocations and depends on the `alloc` crate. Fonts are fully parsed on creation and relevant information is stored in a more convenient to access format. Unlike other font libraries, the font structures have no lifetime dependencies since it allocates its own space.

## Example

[Live demo](https://mooman219.github.io/fontdue/). This demo is a web-assembly build of `fontdue` rasterizing to a browser canvas. It provides a side by side of characters being rasterized between `fontdue` and the browser's canvas text api provided with the same parameters.

Some other examples can be found under ./dev/examples.

### Rasterization
The rasterization API should not see major changes in the near future.
```rust
// Read the font data.
let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];
// Parse it into the font type.
let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
// Rasterize and get the layout metrics for the letter 'g' at 17px.
let (metrics, bitmap) = font.rasterize('g', 17.0);
```

### Layout
The layout API is immature and may see breaking changes. The layout `fontdue` provides is naïve and is only designed to be on par with existing libraries like `glyph_brush`.
```rust
// Read the font data.
let font = include_bytes!("../resources/fonts/Roboto-Regular.ttf") as &[u8];
// Parse it into the font type.
let roboto_regular = Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
// The list of fonts that will be used during layout.
let fonts = &[roboto_regular];
// Create a layout context. Laying out text needs some heap allocations; reusing this context
// reduces the need to reallocate space. We inform layout of which way the Y axis points here.
let mut layout = Layout::new(CoordinateSystem::PositiveYUp);
// By default, layout is initialized with the default layout settings. This call is redundant, but
// demonstrates setting the value with your custom settings.
layout.reset(&LayoutSettings {
    ..LayoutSettings::default()
});
// The text that will be laid out, its size, and the index of the font in the font list to use for
// that section of text.
layout.append(fonts, &TextStyle::new("Hello ", 35.0, 0));
layout.append(fonts, &TextStyle::new("world!", 40.0, 0));
// Prints the layout for "Hello world!"
println!("{:?}", layout.glyphs());

// If you wanted to attached metadata based on the TextStyle to the glyphs returned in the
// glyphs() function, you can use the TextStyle::with_metadata function. In this example, the
// Layout type is now parameterized with u8 (Layout<u8>). All styles need to share the same
// metadata type.
let mut layout = Layout::new(CoordinateSystem::PositiveYUp);
layout.append(fonts, &TextStyle::with_user_data("Hello ", 35.0, 0, 10u8));
layout.append(fonts, &TextStyle::with_user_data("world!", 40.0, 0, 20u8));
println!("{:?}", layout.glyphs());
```

## Performance

### Rasterization

These benchmarks measure the time it takes to generate the glyph metrics and bitmap for the text "Sphinx of black quartz, judge my vow." over a range of sizes. The lower the line in the graph the better.

![Rasterize benchmarks](/images/rasterize_glyf.png)

![Rasterize benchmarks](/images/rasterize_cff.png)

### Layout

This benchmark measures the time it takes to layout latin characters of sample text with wrapping on word boundaries.

![Layout benchmarks](/images/layout.png)

## License

Licensed under any one of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
 * Zlib license ([LICENSE-ZLIB](LICENSE-ZLIB) or http://opensource.org/licenses/Zlib)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be multi-licensed as above, without any additional terms or conditions.

## Notices

### Maintenance

Please bear with me on new features or quirks that you find. Bugs will take priority, but I don't have as much time as I would like to work on fontdue so please be patient, this is a solo project.

### TrueType & OpenType Table Support

Fontdue depends on `ttf-parser` ([link](https://github.com/RazrFalcon/ttf-parser)) for parsing fonts, which supports a wide range of TrueType and OpenType features.

### Attribution

`Fontdue` started as a vaguely more production ready fork of `font-rs` [(link)](https://github.com/raphlinus/font-rs) because of how fast it made rasterization look, and how simple the `rusttype` [(link)](https://gitlab.redox-os.org/redox-os/rusttype) crate made font parsing look. Since then, I've read a lot of font specification and modern rasterization techniques, rewriting `fontdue` from the ground up in the process into its own unique beast.
