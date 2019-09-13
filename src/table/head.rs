use crate::parse::*;
use crate::FontResult;

// Apple: https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6head.html
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/head

pub struct TableHead {
    pub version: u32,
    pub font_revision: u32,
    pub checksum_adjustment: u32,
    pub magic_number: u32,
    pub flags: u16,
    pub units_per_em: u16,
    pub created: i64,
    pub modified: i64,
    pub xmin: i16,
    pub ymin: i16,
    pub xmax: i16,
    pub ymax: i16,
    pub index_to_loc_format: i16,
    pub glyph_data_format: i16,
}

impl TableHead {
    pub fn new(head: &[u8]) -> FontResult<TableHead> {
        let version = read_u32(&head[0..]);
        let font_revision = read_u32(&head[4..]);
        let checksum_adjustment = read_u32(&head[8..]);
        let magic_number = read_u32(&head[12..]);
        if magic_number != 0x5F0_F3CF5 {
            return Err("Font.head: Incorrect magic number, is this a font?");
        }
        let flags = read_u16(&head[16..]);
        let units_per_em = read_u16(&head[18..]);
        let created = read_i64(&head[20..]);
        let modified = read_i64(&head[28..]);
        let xmin = read_i16(&head[36..]);
        let ymin = read_i16(&head[38..]);
        let xmax = read_i16(&head[40..]);
        let ymax = read_i16(&head[42..]);
        let index_to_loc_format = read_i16(&head[50..]);
        let glyph_data_format = read_i16(&head[52..]);
        Ok(TableHead {
            version,
            font_revision,
            checksum_adjustment,
            magic_number,
            flags,
            units_per_em,
            created,
            modified,
            xmin,
            ymin,
            xmax,
            ymax,
            index_to_loc_format,
            glyph_data_format,
        })
    }
}
