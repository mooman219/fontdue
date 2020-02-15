use fontdue::{Font, FontSettings, Format};

const CHAR_SIZE: f32 = 17.0;

const ROBOTO_REGULAR_TTF: &[u8] = include_bytes!("../resources/Roboto-Regular.ttf");
const ROBOTO_MONO_REGULAR_TTF: &[u8] = include_bytes!("../resources/RobotoMono-Regular.ttf");
const LIBERATION_SERIF_REGULAR: &[u8] = include_bytes!("../resources/LiberationSerif-Regular.ttf");

// Performs some basic asserts on the rasterization output.
fn check_best_guess_rasterization((metrics, bitmap): (fontdue::Metrics, Vec<u8>), rendered_char: char, bitdepth: usize) {
    // Ensure this encompasses non-zero area
    assert!(metrics.width > 0, "width must be non-zero for character '{}'", rendered_char);
    assert!(metrics.height > 0, "height must be non-zero for character '{}'", rendered_char);
    // Ensure that the bitmap dimensions matches the metrics' description
    assert_eq!(
        metrics.width * metrics.height * (bitdepth / 8),
        bitmap.len(),
        "bitmap must match dimensions for character '{}'",
        rendered_char
    );
    // Ensure that somewhere in the bitmap is a non-zero pixel
    assert!(bitmap.iter().any(|b| *b != 0), "some pixel must be non-zero when rendering '{}'", rendered_char);
}

#[test]
fn render_roboto_characters() {
    let font = Font::from_bytes(ROBOTO_REGULAR_TTF, FontSettings::default()).unwrap();

    for chr in &['a', '1', '2', '#', '?', '█', '▒', '¾', 'æ'] {
        check_best_guess_rasterization(font.rasterize(*chr, CHAR_SIZE), *chr, 8);
    }
}

#[test]
fn render_roboto_mono_characters() {
    let font = Font::from_bytes(ROBOTO_MONO_REGULAR_TTF, FontSettings::default()).unwrap();

    for chr in &['a', '1', '2', '#', '?', 'æ'] {
        check_best_guess_rasterization(font.rasterize(*chr, CHAR_SIZE), *chr, 8);
    }
}

#[test]
fn render_roboto_mono_characters_rgb() {
    let font = Font::from_bytes(ROBOTO_MONO_REGULAR_TTF, FontSettings{ buffer_kind: Format::Rgb24, flip_vertical: false }).unwrap();

    for chr in &['a', '1', '2', '#', '?', 'æ'] {
        check_best_guess_rasterization(font.rasterize(*chr, CHAR_SIZE), *chr, 24);
    }
}

#[test]
fn render_roboto_mono_characters_rgba() {
    let font = Font::from_bytes(ROBOTO_MONO_REGULAR_TTF, FontSettings{ buffer_kind: Format::Rgba32, flip_vertical: false }).unwrap();

    for chr in &['a', '1', '2', '#', '?', 'æ'] {
        check_best_guess_rasterization(font.rasterize(*chr, CHAR_SIZE), *chr, 32);
    }
}

#[test]
fn render_liberation_serif_characters() {
    let font = Font::from_bytes(LIBERATION_SERIF_REGULAR, FontSettings::default()).unwrap();

    for chr in &['a', '1', '2', '#', '?', '█', '▒', '¾', 'æ'] {
        check_best_guess_rasterization(font.rasterize(*chr, CHAR_SIZE), *chr, 8);
    }
}
