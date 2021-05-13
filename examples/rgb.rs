//! Generates unfiltered subpixel RGB output for Fontdue.

use std::fs::File;
use std::io::Write;

// Scratch pad for glyphs: ⅞ g
const CHARACTER: char = 'g';
const SIZE: f32 = 12.0;

// cargo run --example rgb --release
pub fn main() {
    // Loading and rasterization
    let font = include_bytes!("../resources/fonts/Roboto-Regular.ttf") as &[u8];
    let settings = fontdue::FontSettings {
        scale: SIZE,
        ..fontdue::FontSettings::default()
    };
    let font = fontdue::Font::from_bytes(font, settings).unwrap();
    let (metrics, bitmap) = font.rasterize_subpixel(CHARACTER, SIZE);

    // Output
    let mut o = File::create("rgb.ppm").unwrap();
    let _ = o.write(format!("P6\n{} {}\n255\n", metrics.width, metrics.height).as_bytes());
    let _ = o.write(&bitmap);
}
