use fontdue::raw::*;

pub fn main() {
    let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];
    let font = RawFont::new(font).unwrap();
    println!("head.units_per_em: {}", font.head.units_per_em);
    println!("head.xmin: {}", font.head.xmin);
    println!("head.ymin: {}", font.head.ymin);
    println!("head.xmax: {}", font.head.xmax);
    println!("head.ymax: {}", font.head.ymax);
    println!("maxp.num_glyphs: {}", font.maxp.num_glyphs);
    println!("cmap.map: {}", font.cmap.map.len());
    println!("head.index_to_loc_format: {}", font.head.index_to_loc_format);

    if let Some(hmtx) = &font.hmtx {
        println!("hmtx.hmetrics: {}", hmtx.hmetrics.len());
    }

    if let Some(hhea) = &font.hhea {
        println!("hhea.ascent: {}", hhea.ascent);
        println!("hhea.descent: {}", hhea.descent);
        println!("hhea.line_gap: {}", hhea.line_gap);
        let new_line = (hhea.ascent - hhea.descent + hhea.line_gap) as f32;
        println!("hhea.new_line_height: {}", new_line);
    }

    println!("{:?}", font.maxp);
    // println!("{:?}", font.hmtx);
}
