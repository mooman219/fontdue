use fontdue::raw::*;

pub fn main() {
    let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];
    let font = RawFont::new(font).unwrap();
    // Letter lookup code.
    let glyph = font.glyf.glyphs[298].clone();
    println!("Total points: {}", glyph.points.len());
    for point in &glyph.points {
        println!("{:?}", point);
    }
}
