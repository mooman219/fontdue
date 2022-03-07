//! Generates grayscale output for Fontdue. There are also examples for doing a similar operation in
//! Rusttype and FreeType.

use std::fs::File;
use std::io::Write;

// Scratch pad for glyphs: ⅞ g ₹
const CHARACTER: char = '₹';
const SIZE: f32 = 200.0;
const PATH: &[u8] = include_bytes!("../resources/fonts/GreatVibes-Regular.otf") as &[u8];

// cargo run --example simple --release
pub fn main() {
    generate_fontdue();
}

pub fn generate_fontdue() {
    // Read the font data.
    let font = PATH;
    // Setup the configuration for how the font will be parsed.
    let settings = fontdue::FontSettings {
        scale: SIZE,
        ..fontdue::FontSettings::default()
    };
    // Parse it into the font type.
    let font = fontdue::Font::from_bytes(font, settings).unwrap();
    // Rasterize and get the layout metrics for the character at a size.
    let (metrics, bitmap) = font.rasterize(CHARACTER, SIZE);

    // Output
    let mut o = File::create("fontdue.pgm").unwrap();
    let _ = o.write(format!("P5\n{} {}\n255\n", metrics.width, metrics.height).as_bytes());
    let _ = o.write(&bitmap);
}

pub fn generate_rusttype() {
    // Loading and rasterization
    let font = PATH;
    let font = rusttype::Font::try_from_bytes(font).unwrap();
    let glyph =
        font.glyph(CHARACTER).scaled(rusttype::Scale::uniform(SIZE)).positioned(rusttype::point(0.0, 0.0));
    let (height, width) = if let Some(rect) = glyph.pixel_bounding_box() {
        (rect.height(), rect.width())
    } else {
        (0, 0)
    };
    let mut bitmap = vec![0u8; (width * height) as usize];
    glyph.draw(|x, y, v| {
        bitmap[(x as usize) + (y as usize) * width as usize] = (v * 255.0) as u8;
    });

    // Output
    let mut o = File::create("rusttype.pgm").unwrap();
    let _ = o.write(format!("P5\n{} {}\n255\n", width, height).as_bytes());
    let _ = o.write(&bitmap);
}

#[cfg(feature = "freetype_benchmark")]
pub fn generate_freetype() {
    use freetype::Library;

    // Loading and rasterization
    let font = PATH;
    let lib = Library::init().unwrap();
    let font = font.to_vec();
    let face = lib.new_memory_face(font, 0).unwrap();
    face.set_char_size(0, (SIZE * 0.75 * 64.0) as isize, 96, 96).unwrap();
    face.load_char(CHARACTER as usize, freetype::face::LoadFlag::RENDER).unwrap();
    let glyph = face.glyph();
    let bitmap = glyph.bitmap();

    // Output
    let mut o = File::create("freetype.pgm").unwrap();
    let _ = o.write(format!("P5\n{} {}\n255\n", bitmap.width(), bitmap.rows()).as_bytes());
    let _ = o.write(bitmap.buffer());
}
