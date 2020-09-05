use fontdue::layout::{Layout, LayoutSettings, TextStyle};

// cargo run --example layout --release
pub fn main() {
    let font = include_bytes!("../resources/fonts/Roboto-Regular.ttf") as &[u8];
    let roboto_regular = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
    let mut layout = Layout::new();
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
}
