use crate::parse::*;
use crate::FontResult;

// Apple: https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6head.html
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/head

#[derive(Debug, PartialEq)]
pub struct TableHead {
    pub version_major: u16,
    pub version_minor: u16,
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
    pub mac_style: u16,
    pub lowest_rec_ppem: u16,
    pub font_direction_hint: i16,
    pub index_to_loc_format: i16,
    pub glyph_data_format: i16,
}

impl TableHead {
    pub fn new(head: &[u8]) -> FontResult<TableHead> {
        let mut stream = Stream::new(head);
        let version_major = stream.read_u16();
        let version_minor = stream.read_u16();
        let font_revision = stream.read_u32();
        let checksum_adjustment = stream.read_u32();
        let magic_number = stream.read_u32();
        if magic_number != 0x5F0_F3CF5 {
            return Err("Font.head: Incorrect magic number, is this a font?");
        }
        let flags = stream.read_u16();
        let units_per_em = stream.read_u16();
        let created = stream.read_i64();
        let modified = stream.read_i64();
        let xmin = stream.read_i16();
        let ymin = stream.read_i16();
        let xmax = stream.read_i16();
        let ymax = stream.read_i16();
        let mac_style = stream.read_u16();
        let lowest_rec_ppem = stream.read_u16();
        let font_direction_hint = stream.read_i16();
        let index_to_loc_format = stream.read_i16();
        let glyph_data_format = stream.read_i16();
        Ok(TableHead {
            version_major,
            version_minor,
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
            mac_style,
            lowest_rec_ppem,
            font_direction_hint,
            index_to_loc_format,
            glyph_data_format,
        })
    }
}
