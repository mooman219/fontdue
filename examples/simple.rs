use rusttype::*;
use std::fs::File;
use std::io::Write;

// Scratch pad for glyphs: ⅞ g
const CHARACTER: char = '⅞';
const SIZE: f32 = 17.0;
// let (metrics, bitmap) = font.rasterize(CHARACTER, SIZE - 2.0);

pub fn main() {
    // Loading and rasterization
    let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];
    let mut font = fontdue::Font::from_bytes(font).unwrap();
    let (metrics, bitmap) = font.rasterize('g', 17.0);

    // Output
    let mut o = File::create("fontdue.pgm").unwrap();
    let _ = o.write(format!("P5\n{} {}\n255\n", metrics.width, metrics.height).as_bytes());
    let _ = o.write(&bitmap);

    let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];
    let font = rusttype::Font::from_bytes(font).unwrap();
    let glyph = font.glyph(CHARACTER).scaled(Scale::uniform(SIZE)).positioned(rusttype::point(0.0, 0.0));
    let rect = glyph.pixel_bounding_box().unwrap();
    let mut bitmap = vec![0u8; (rect.width() * rect.height()) as usize];
    glyph.draw(|x, y, v| {
        bitmap[(x as usize) + (y as usize) * rect.width() as usize] = (v * 255.0) as u8;
    });

    let mut o = File::create("rusttype.pgm").unwrap();
    let _ = o.write(format!("P5\n{} {}\n255\n", rect.width(), rect.height()).as_bytes());
    let _ = o.write(&bitmap);
}
