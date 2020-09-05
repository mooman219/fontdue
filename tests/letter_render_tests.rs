use fontdue::{Font, FontSettings};

const SMALL_SIZES: [f32; 5] = [100.0, 32.0, 16.0, 4.0, 2.0];
const MED_SIZES: [f32; 2] = [1024.0, 500.0];
const LARGE_SIZES: [f32; 1] = [2048.0];
const CHARACTERS: [char; 94] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u',
    'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
    'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '!',
    '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '-', '=', '`', '~', '[', ']', '\\', ';', '\'',
    ',', '.', '/', '{', '}', '|', ':', '"', '<', '>', '?',
];
static FONTS: [&[u8]; 3] = [
    include_bytes!("../resources/fonts/Roboto-Regular.ttf"),
    include_bytes!("../resources/fonts/RobotoMono-Regular.ttf"),
    include_bytes!("../resources/fonts/Exo2-Regular.otf"),
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

fn render_characters(sizes: &[f32]) {
    for font in &FONTS {
        let font = Font::from_bytes(*font, FontSettings::default()).unwrap();
        for character in CHARACTERS.iter().copied() {
            for size in sizes {
                check_best_guess_rasterization(font.rasterize(character, *size), character);
            }
        }
    }
}

#[test]
fn render_characters_small() {
    render_characters(&SMALL_SIZES);
}

#[test]
fn render_characters_med() {
    render_characters(&MED_SIZES);
}

#[test]
fn render_characters_large() {
    render_characters(&LARGE_SIZES);
}
