use fontdue::{Font, FontSettings};

const SIZES: [f32; 6] = [1024.0, 100.0, 32.0, 16.0, 4.0, 2.0];
const CHARACTERS: [char; 94] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u',
    'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
    'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '!',
    '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '+', '-', '=', '`', '~', '[', ']', '\\', ';', '\'',
    ',', '.', '/', '{', '}', '|', ':', '"', '<', '>', '?',
];
static FONTS: [&[u8]; 8] = [
    include_bytes!("../resources/fonts/Roboto-Regular.ttf"),
    include_bytes!("../resources/fonts/RobotoMono-Regular.ttf"),
    include_bytes!("../resources/fonts/Comfortaa-Regular.ttf"),
    include_bytes!("../resources/fonts/Inconsolata-Regular.ttf"),
    include_bytes!("../resources/fonts/FasterOne-Regular.ttf"),
    include_bytes!("../resources/fonts/Exo2-Regular.otf"),
    include_bytes!("../resources/fonts/GreatVibes-Regular.otf"),
    include_bytes!("../resources/fonts/modernpics.otf"),
];

// Performs some basic asserts on the rasterization output.
fn check_best_guess_rasterization(
    (metrics, bitmap): (fontdue::Metrics, Vec<u8>),
    rendered_char: char,
    index: usize,
) {
    // Ensure that the bitmap dimensions matches the metrics' description
    assert_eq!(
        metrics.width * metrics.height,
        bitmap.len(),
        "bitmap must match dimensions for character [{}] at index [{}]",
        rendered_char,
        index
    );
    if bitmap.len() > 0 {
        let mut visible = false;
        for x in bitmap {
            if x > 0 {
                visible = true;
                break;
            }
        }
        assert!(visible, "No valid bitmap for [{}] at index [{}]", rendered_char, index);
    }
}

fn render_common(sizes: &[f32]) {
    for font in &FONTS {
        let font = Font::from_bytes(*font, FontSettings::default()).unwrap();
        println!("Rendering characters for: {:?}", font);
        for character in CHARACTERS.iter().copied() {
            for size in sizes {
                check_best_guess_rasterization(
                    font.rasterize(character, *size),
                    character,
                    font.lookup_glyph_index(character),
                );
            }
        }
    }
}

#[test]
fn render_all_small() {
    for font in &FONTS {
        let font = Font::from_bytes(*font, FontSettings::default()).unwrap();
        println!("Rendering characters for: {:?}", font);
        for index in 0..font.glyph_count() {
            check_best_guess_rasterization(font.rasterize_indexed(index, 8.0), ' ', index);
        }
    }
}

#[test]
fn render_common_scaled() {
    render_common(&SIZES);
}
