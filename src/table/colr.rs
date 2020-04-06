use alloc::vec::*;
use crate::parse::*;
use crate::FontResult;

// Multi-colored glyph layer information, requires the CPAL table to be present
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/colr

#[derive(Debug)]
pub struct TableColr {
    pub header: Header,
    pub base_glyph_records: Vec<BaseGlyphRecord>,
    pub layer_records: Vec<LayerRecord>,
}

#[derive(Copy, Clone, Debug)]
pub struct Header {
    pub version: u16,
    pub num_base_glyph_records: u16,
    pub base_glyph_records_offset: u32,
    pub layer_records_offset: u32,
    pub num_layer_records: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct BaseGlyphRecord {
    pub gid: u16,
    pub first_layer_index: u16,
    pub num_layers: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct LayerRecord {
    pub gid: u16,
    pub palette_index: u16,
}

impl TableColr  {
    pub fn new(colr: &[u8]) -> FontResult<TableColr> {
        let mut stream = Stream::new(colr);
        let header = Self::read_header(&mut stream);
        let base_glyph_records = Self::read_base_glyph_records(&mut stream, header.base_glyph_records_offset, header.num_base_glyph_records);
        let layer_records = Self::read_layer_records(&mut stream, header.layer_records_offset, header.num_layer_records);
        Ok(TableColr {
            header,
            base_glyph_records,
            layer_records,
        })
    }

    fn read_header(stream: &mut Stream) -> Header {
        let version = stream.read_u16();
        let num_base_glyph_records = stream.read_u16();
        let base_glyph_records_offset = stream.read_u32();
        let layer_records_offset = stream.read_u32();
        let num_layer_records = stream.read_u16();

        Header {
            version,
            num_base_glyph_records,
            base_glyph_records_offset,
            layer_records_offset,
            num_layer_records,
        }
    }

    fn read_base_glyph_records(stream: &mut Stream, base_glyph_records_offset: u32, num_base_glyph_records: u16) -> Vec<BaseGlyphRecord> {
        stream.seek(base_glyph_records_offset as usize);
        let mut result = Vec::with_capacity(num_base_glyph_records as usize);
        for _ in 0..num_base_glyph_records {
            result.push(BaseGlyphRecord {
                gid: stream.read_u16(),
                first_layer_index: stream.read_u16(),
                num_layers: stream.read_u16(),
            });
        }
        result
    }

    fn read_layer_records(stream: &mut Stream, layer_records_offset: u32, num_layer_records: u16) -> Vec<LayerRecord> {
        stream.seek(layer_records_offset as usize);
        let mut result = Vec::with_capacity(num_layer_records as usize);
        for _ in 0..num_layer_records {
            result.push(LayerRecord {
                gid: stream.read_u16(),
                palette_index: stream.read_u16(),
            });
        }
        result
    }
}
