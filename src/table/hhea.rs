use crate::parse::*;
use crate::FontResult;

// Apple: https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6hhea.html
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/hhea

pub struct TableHhea {
    pub version: u32,
    pub ascent: i16,
    pub descent: i16,
    pub line_gap: i16,
    pub advance_width_max: u16,
    pub min_left_side_bearing: i16,
    pub min_right_side_bearing: i16,
    pub xmax_extent: i16,
    pub caret_slope_rise: i16,
    pub caret_slope_run: i16,
    pub caret_offset: i16,
    pub num_long_hmetrics: u16,
}

impl TableHhea {
    pub fn new(hhea: &[u8]) -> FontResult<TableHhea> {
        let version = read_u32(&hhea[0..]);
        let ascent = read_i16(&hhea[4..]);
        let descent = read_i16(&hhea[6..]);
        let line_gap = read_i16(&hhea[8..]);
        let advance_width_max = read_u16(&hhea[10..]);
        let min_left_side_bearing = read_i16(&hhea[12..]);
        let min_right_side_bearing = read_i16(&hhea[14..]);
        let xmax_extent = read_i16(&hhea[16..]);
        let caret_slope_rise = read_i16(&hhea[18..]);
        let caret_slope_run = read_i16(&hhea[20..]);
        let caret_offset = read_i16(&hhea[22..]);
        let num_long_hmetrics = read_u16(&hhea[34..]);
        if num_long_hmetrics == 0 {
            return Err("Font.hhea: The number of long hmetrics must be geater than 0");
        }
        Ok(TableHhea {
            version,
            ascent,
            descent,
            line_gap,
            advance_width_max,
            min_left_side_bearing,
            min_right_side_bearing,
            xmax_extent,
            caret_slope_rise,
            caret_slope_run,
            caret_offset,
            num_long_hmetrics,
        })
    }
}
