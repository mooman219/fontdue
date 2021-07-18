//! Performs basic text layout in Fontdue.

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use fontdue::Font;

// cargo run --example layout --release
pub fn main() {
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
    // glyphs() function, you can use the TextStyle::with_user_data function. In this example, the
    // Layout type is now parameterized with u8 (Layout<u8>). All styles need to share the same
    // metadata type.
    let mut layout = Layout::new(CoordinateSystem::PositiveYUp);
    layout.append(fonts, &TextStyle::with_user_data("Hello ", 35.0, 0, 10u8));
    layout.append(fonts, &TextStyle::with_user_data("world!", 40.0, 0, 20u8));
    println!("{:?}", layout.glyphs());
}
