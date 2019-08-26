use fontdue::raw::*;

pub fn main() {
    let font = include_bytes!("resources/Roboto-Regular.ttf") as &[u8];
    let font = RawFont::new(font).unwrap();
    println!("head.units_per_em: {}", font.head.units_per_em);
    println!("head.xmin: {}", font.head.xmin);
    println!("head.ymin: {}", font.head.ymin);
    println!("head.xmax: {}", font.head.xmax);
    println!("head.ymax: {}", font.head.ymax);
    println!("maxp.num_glyphs: {}", font.maxp.num_glyphs);
    println!("hmtx.hmetrics: {}", font.hmtx.hmetrics.len());
    println!("cmap.map: {}", font.cmap.map.len());
    println!("head.index_to_loc_format: {}", font.head.index_to_loc_format);
    for point in &font.glyf.glyphs[47].points {
        println!("{:?}", point);
    }
}
