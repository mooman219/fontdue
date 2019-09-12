use fontdue::*;
use std::fs::File;
use std::io::Write;

pub fn main() {
    // Loading and rasterization
    let font = include_bytes!("resources/Roboto-Regular.ttf") as &[u8];
    let font = Font::from_bytes(font).unwrap();
    // Scratch pad for glyphs: ⅞ g
    let (metrics, bitmap) = font.rasterize('g', 17.0);

    // Output
    let mut o = File::create("simple.pgm").unwrap();
    let _ = o.write(format!("P5\n{} {}\n255\n", metrics.width, metrics.height).as_bytes());
    let _ = o.write(&bitmap);
}
