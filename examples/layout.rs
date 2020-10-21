use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};

#[derive(Debug)]
struct UserData {
    color: (f32, f32, f32, f32),
}

// cargo run --example layout --release
pub fn main() {
    let font = include_bytes!("../resources/fonts/Roboto-Regular.ttf") as &[u8];
    let roboto_regular = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
    let mut layout = Layout::new(CoordinateSystem::PositiveYUp);
    let mut output = Vec::new();
    let settings = LayoutSettings {
        ..LayoutSettings::default()
    };
    let fonts = &[roboto_regular];
    let styles = &[&TextStyle::new("Hello ", 35.0, 0), &TextStyle::new("world!", 40.0, 0)];
    layout.layout_horizontal(fonts, styles, &settings, &mut output);

    for glyph in output {
        println!("{:?}", glyph);
    }

    // Once more, with user data
    println!();
    println!("User data version:");

    let styles = &[
        &TextStyle::with_user_data(
            "Hello ",
            35.0,
            0,
            UserData {
                color: (1.0, 1.0, 1.0, 1.0),
            },
        ),
        &TextStyle::with_user_data(
            "world!",
            40.0,
            0,
            UserData {
                color: (0.0, 0.0, 0.0, 0.0),
            },
        ),
    ];

    let mut output = Vec::new();
    layout.layout_horizontal(fonts, styles, &settings, &mut output);

    for glyph in output {
        println!("{:?}", glyph);
    }
}
