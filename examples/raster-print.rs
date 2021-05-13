//! Generates grayscale and unfiltered subpixel RGB output for Fontdue in the terminal.

const CHARACTER: char = 'b';
const SIZE: f32 = 20.0;

// cargo run --example raster-print --release
pub fn main() {
    let font = include_bytes!("../resources/fonts/Roboto-Regular.ttf") as &[u8];
    let settings = fontdue::FontSettings {
        scale: SIZE,
        ..fontdue::FontSettings::default()
    };
    let font = fontdue::Font::from_bytes(font, settings).unwrap();

    println!("Normal:");
    print_normal(&font);
    println!("\nSubpixel:");
    print_subpixel(&font);
}

pub fn print_normal(font: &fontdue::Font) {
    let (metrics, bitmap) = font.rasterize(CHARACTER, SIZE);
    for y in 0..metrics.height {
        for x in 0..metrics.width {
            let char_s = bitmap[x + y * metrics.width];
            print!("\x1B[48;2;{};{};{}m   ", char_s, char_s, char_s);
        }
        println!("\x1B[0m");
    }
}

pub fn print_subpixel(font: &fontdue::Font) {
    let (metrics, bitmap) = font.rasterize_subpixel(CHARACTER, SIZE);
    for y in 0..metrics.height {
        for x in (0..metrics.width * 3).step_by(3) {
            let char_r = bitmap[x + y * metrics.width * 3];
            let char_g = bitmap[x + 1 + y * metrics.width * 3];
            let char_b = bitmap[x + 2 + y * metrics.width * 3];
            print!("\x1B[48;2;{};{};{}m   ", char_r, char_g, char_b);
        }
        println!("\x1B[0m");
    }
}
