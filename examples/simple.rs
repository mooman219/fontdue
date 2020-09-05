use std::fs::File;
use std::io::Write;

// Scratch pad for glyphs: ⅞ g
const CHARACTER: char = 'd';
const SIZE: f32 = 500.0;

// cargo run --example simple --release
pub fn main() {
    // Loading and rasterization
    let font = include_bytes!("../resources/fonts/Roboto-Regular.ttf") as &[u8];
    let settings = fontdue::FontSettings {
        scale: SIZE,
        ..fontdue::FontSettings::default()
    };
    let font = fontdue::Font::from_bytes(font, settings).unwrap();
    let (metrics, bitmap) = font.rasterize(CHARACTER, SIZE);

    // Output
    let mut o = File::create("fontdue.pgm").unwrap();
    let _ = o.write(format!("P5\n{} {}\n255\n", metrics.width, metrics.height).as_bytes());
    let _ = o.write(&bitmap);
}
