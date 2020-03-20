use fontdue::{Font, FontSettings};

const SIZES: [f32; 7] = [1024.0, 1000.0, 500.0, 32.0, 16.0, 4.0, 3.0];
const CHARACTERS: [char; 9] = ['a', '1', '2', '#', '?', '█', '▒', '¾', 'æ'];
const FONTS: [&[u8]; 3] = [
    include_bytes!("../resources/Roboto-Regular.ttf"),
    include_bytes!("../resources/RobotoMono-Regular.ttf"),
    include_bytes!("../resources/LiberationSerif-Regular.ttf"),
];

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
fn render_characters() {
    for font in &FONTS {
        let font = Font::from_bytes(*font, FontSettings::default()).unwrap();
        for character in &CHARACTERS {
            for size in &SIZES {
                check_best_guess_rasterization(font.rasterize(*character, *size), *character);
            }
        }
    }
}
