use fontdue::{Font, FontSettings};

const SMALL_SIZES: [f32; 5] = [128.0, 32.0, 16.0, 4.0, 3.0];
const LARGE_SIZES: [f32; 3] = [2048.0, 1024.0, 512.0];
const CHARACTERS: [char; 94] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u',
    'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
    'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '!',
    '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '-', '=', '`', '~', '[', ']', '\\', ';', '\'',
    ',', '.', '/', '{', '}', '|', ':', '"', '<', '>', '?',
];
static FONTS: [&[u8]; 3] = [
    include_bytes!("../resources/Roboto-Regular.ttf"),
    include_bytes!("../resources/RobotoMono-Regular.ttf"),
    include_bytes!("../resources/LiberationSerif-Regular.ttf"),
];

// Performs some basic asserts on the rasterization output.
fn check_best_guess_rasterization((metrics, bitmap): (fontdue::Metrics, Vec<u8>), rendered_char: char) {
    // Ensure that the bitmap dimensions matches the metrics' description
    assert_eq!(
        metrics.width * metrics.height,
        bitmap.len(),
        "bitmap must match dimensions for character '{}'",
        rendered_char
    );
}

#[test]
fn render_characters_small() {
    for font in &FONTS {
        let font = Font::from_bytes(*font, FontSettings::default()).unwrap();
        for character in CHARACTERS.iter().copied() {
            for size in &SMALL_SIZES {
                check_best_guess_rasterization(font.rasterize(character, *size), character);
            }
        }
    }
}

#[test]
fn render_characters_large() {
    for font in &FONTS {
        let font = Font::from_bytes(*font, FontSettings::default()).unwrap();
        for character in CHARACTERS.iter().copied() {
            for size in &LARGE_SIZES {
                check_best_guess_rasterization(font.rasterize(character, *size), character);
            }
        }
    }
}
