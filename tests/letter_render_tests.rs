use fontdue;
use std::assert;
use std::fs;
use std::path::Path;

const CHAR_SIZE: f32 = 17.0;

// Loads a font by file name from the `resources/` directory
fn load_font(font_file: String) -> fontdue::Font {
    let file_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("resources").join(font_file);
    let font = fs::read(file_path).unwrap();
    return fontdue::Font::from_bytes(font).unwrap();
}

// Performs some basic asserts on the rasterization output.
fn check_best_guess_rasterization((metrics, bitmap): (fontdue::Metrics, Vec<u8>), rendered_char: char) {
    // Ensure this encompasses non-zero area
    assert!(metrics.width > 0, "width must be non-zero for character '{}'", rendered_char);
    assert!(metrics.height > 0, "height must be non-zero for character '{}'", rendered_char);
    // Ensure that the bitmap dimensions matches the metrics' description
    assert_eq!(
        metrics.width * metrics.height,
        bitmap.len(),
        "bitmap must match dimensions for character '{}'",
        rendered_char
    );
    // Ensure that somewhere in the bitmap is a non-zero pixel
    assert!(bitmap.iter().any(|b| *b != 0), "some pixel must be non-zero when rendering '{}'", rendered_char);
}

#[test]
fn render_roboto_characters() {
    let font = load_font(String::from("Roboto-Regular.ttf"));

    for chr in &['a', '1', '2', '#', '?', '█', '▒', '¾', 'æ'] {
        check_best_guess_rasterization(font.rasterize(*chr, CHAR_SIZE), *chr);
    }
}

#[test]
fn render_roboto_mono_characters() {
    let font = load_font(String::from("RobotoMono-Regular.ttf"));

    for chr in &['a', '1', '2', '#', '?', 'æ'] {
        check_best_guess_rasterization(font.rasterize(*chr, CHAR_SIZE), *chr);
    }
}

#[test]
fn render_liberation_serif_characters() {
    let font = load_font(String::from("LiberationSerif-Regular.ttf"));

    for chr in &['a', '1', '2', '#', '?', '█', '▒', '¾', 'æ'] {
        check_best_guess_rasterization(font.rasterize(*chr, CHAR_SIZE), *chr);
    }
}
