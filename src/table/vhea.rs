use crate::parse::*;
use crate::FontResult;

// Apple: https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6vhea.html
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/vhea

pub struct TableVhea {
    pub version: u32,
    pub ascent: i16,
    pub descent: i16,
    pub line_gap: i16,
    pub advance_height_max: u16,
    pub min_top_side_bearing: i16,
    pub min_bottom_side_bearing: i16,
    pub ymax_extent: i16,
    pub caret_slope_rise: i16,
    pub caret_slope_run: i16,
    pub caret_offset: i16,
    pub num_long_vmetrics: u16,
}

impl TableVhea {
    pub fn new(vhea: &[u8]) -> FontResult<TableVhea> {
        let version = read_u32(&vhea[0..]);
        let ascent = read_i16(&vhea[4..]);
        let descent = read_i16(&vhea[6..]);
        let line_gap = read_i16(&vhea[8..]);
        let advance_height_max = read_u16(&vhea[10..]);
        let min_top_side_bearing = read_i16(&vhea[12..]);
        let min_bottom_side_bearing = read_i16(&vhea[14..]);
        let ymax_extent = read_i16(&vhea[16..]);
        let caret_slope_rise = read_i16(&vhea[18..]);
        let caret_slope_run = read_i16(&vhea[20..]);
        let caret_offset = read_i16(&vhea[22..]);
        let num_long_vmetrics = read_u16(&vhea[34..]);
        if num_long_vmetrics == 0 {
            return Err("Font.vhea: The number of long hmetrics must be geater than 0");
        }
        Ok(TableVhea {
            version,
            ascent,
            descent,
            line_gap,
            advance_height_max,
            min_top_side_bearing,
            min_bottom_side_bearing,
            ymax_extent,
            caret_slope_rise,
            caret_slope_run,
            caret_offset,
            num_long_vmetrics,
        })
    }
}
