use std::fs::File;
use std::io::Write;

// Scratch pad for glyphs: ⅞ g
const CHARACTER: char = 'g';
const SIZE: f32 = 50.0;

pub fn main() {
    // Loading and rasterization
    let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];
    let font = fontdue::Font::from_bytes(font, fontdue::FontSettings{ flip_vertical: false, buffer_kind: fontdue::Format::Rgba32 }).unwrap();
    let (metrics, bitmap) = font.rasterize(CHARACTER, SIZE);

    // Output
    let mut o = File::create("fontdue.pam").unwrap();
    let _ = o.write(format!("P7\nWIDTH {}\nHEIGHT {}\nDEPTH 4\nMAXVAL 255\nTUPLTYPE RGB_ALPHA\nENDHDR\n", metrics.width, metrics.height).as_bytes());
    let _ = o.write(&bitmap);
}
