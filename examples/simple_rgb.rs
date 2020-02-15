use std::fs::File;
use std::io::Write;

// Scratch pad for glyphs: ⅞ g
const CHARACTER: char = 'g';
const SIZE: f32 = 50.0;

pub fn main() {
    // Loading and rasterization
    let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];
    let font = fontdue::Font::from_bytes(font, fontdue::FontSettings{ flip_vertical: false, buffer_kind: fontdue::Format::Rgb24 }).unwrap();
    let (metrics, bitmap) = font.rasterize(CHARACTER, SIZE);

    // Output
    let mut o = File::create("fontdue.ppm").unwrap();
    let _ = o.write(format!("P6\n{} {}\n255\n", metrics.width, metrics.height).as_bytes());
    let _ = o.write(&bitmap);
}
