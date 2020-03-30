use fontdue::raw::*;

static FONT: &[u8] = include_bytes!("../resources/Roboto-Regular.ttf");

#[test]
fn header_head() {
    let font = RawFont::new(FONT).unwrap();
    assert_eq!(font.head.units_per_em, 2048);
    assert_eq!(font.head.xmin, -1509);
    assert_eq!(font.head.ymin, -555);
    assert_eq!(font.head.xmax, 2352);
    assert_eq!(font.head.ymax, 2163);
    assert_eq!(font.head.index_to_loc_format, 0);
}

#[test]
fn header_maxp() {
    let font = RawFont::new(FONT).unwrap();
    assert_eq!(font.maxp.num_glyphs, 1294);
}

#[test]
fn header_cmap() {
    let font = RawFont::new(FONT).unwrap();
    assert_eq!(font.cmap.map.len(), 896);
}

#[test]
fn header_hmtx() {
    let font = RawFont::new(FONT).unwrap();
    let hmtx = font.hmtx.expect("Missing the expected hmtx table.");
    assert_eq!(hmtx.hmetrics.len(), 1294);
}
