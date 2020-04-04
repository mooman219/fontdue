use fontdue::raw::*;

static FONT: &[u8] = include_bytes!("../resources/Roboto-Regular.ttf");

#[test]
fn header_head() {
    let font = RawFont::new(FONT).unwrap();
    let expected = TableHead {
        version_major: 1,
        version_minor: 0,
        font_revision: 140050,
        checksum_adjustment: 3949969458,
        magic_number: 1594834165,
        flags: 25,
        units_per_em: 2048,
        created: 3304067374,
        modified: 3573633780,
        xmin: -1509,
        ymin: -555,
        xmax: 2352,
        ymax: 2163,
        mac_style: 0,
        lowest_rec_ppem: 9,
        font_direction_hint: 2,
        index_to_loc_format: 0,
        glyph_data_format: 0,
    };
    assert_eq!(font.head, expected);
}

#[test]
fn header_hhea() {
    let font = RawFont::new(FONT).unwrap();
    let hhea = font.hhea.expect("Missing the expected hhea table.");
    let expected = TableHhea {
        version: 65536,
        ascent: 1900,
        descent: -500,
        line_gap: 0,
        advance_width_max: 2377,
        min_left_side_bearing: -1509,
        min_right_side_bearing: -438,
        xmax_extent: 2352,
        caret_slope_rise: 1,
        caret_slope_run: 0,
        caret_offset: 0,
        metric_data_format: 0,
        num_long_hmetrics: 1294,
    };
    assert_eq!(hhea, expected);
}

#[test]
fn header_maxp() {
    let font = RawFont::new(FONT).unwrap();
    let expected = TableMaxp {
        num_glyphs: 1294,
    };
    assert_eq!(font.maxp, expected);
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
