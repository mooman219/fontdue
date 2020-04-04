use crate::parse::*;
use crate::FontResult;

// Apple: https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6vhea.html
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/vhea

#[derive(Debug, PartialEq)]
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
    pub metric_data_format: i16,
    pub num_long_vmetrics: u16,
}

impl TableVhea {
    pub fn new(vhea: &[u8]) -> FontResult<TableVhea> {
        let mut stream = Stream::new(vhea);
        let version = stream.read_u32();
        let ascent = stream.read_i16();
        let descent = stream.read_i16();
        let line_gap = stream.read_i16();
        let advance_height_max = stream.read_u16();
        let min_top_side_bearing = stream.read_i16();
        let min_bottom_side_bearing = stream.read_i16();
        let ymax_extent = stream.read_i16();
        let caret_slope_rise = stream.read_i16();
        let caret_slope_run = stream.read_i16();
        let caret_offset = stream.read_i16();
        stream.skip(8); // Reserved
        let metric_data_format = stream.read_i16();
        let num_long_vmetrics = stream.read_u16();
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
            metric_data_format,
            num_long_vmetrics,
        })
    }
}
