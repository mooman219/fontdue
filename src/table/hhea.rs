use crate::parse::*;
use crate::FontResult;

// Apple: https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6hhea.html
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/hhea

#[derive(Debug, PartialEq)]
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
    pub metric_data_format: i16,
    pub num_long_hmetrics: u16,
}

impl TableHhea {
    pub fn new(hhea: &[u8]) -> FontResult<TableHhea> {
        let mut stream = Stream::new(hhea);
        let version = stream.read_u32();
        let ascent = stream.read_i16();
        let descent = stream.read_i16();
        let line_gap = stream.read_i16();
        let advance_width_max = stream.read_u16();
        let min_left_side_bearing = stream.read_i16();
        let min_right_side_bearing = stream.read_i16();
        let xmax_extent = stream.read_i16();
        let caret_slope_rise = stream.read_i16();
        let caret_slope_run = stream.read_i16();
        let caret_offset = stream.read_i16();
        stream.skip(8); // Reserved
        let metric_data_format = stream.read_i16();
        let num_long_hmetrics = stream.read_u16();
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
            metric_data_format,
            num_long_hmetrics,
        })
    }
}
