use alloc::vec::*;
use crate::parse::*;
use crate::FontResult;

// Color pallete information used by the COLR and sometimes SVG tables
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/cpal

#[derive(Debug)]
pub struct TableCpal {
    pub header: Header,
    pub color_records: Vec<BGRA8Color>
}

#[derive(Copy, Clone, Debug)]
pub struct BGRA8Color {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
}

#[derive(Clone, Debug)]
pub struct Header {
    pub version: u16,
    pub num_palette_entries: u16,
    pub num_palettes: u16,
    pub num_color_records: u16,
    pub offset_first_color_record: u32,
    pub color_record_indicies: Vec<u16>,
}

impl TableCpal {
    pub fn new(cpal: &[u8]) -> FontResult<TableCpal> {
        let mut stream = Stream::new(cpal);

        let version = stream.read_u16();
        let header;
        match version {
            0x0000 | 0x0001 => header = Self::read_header(&mut stream, version),
            _ => return Err("Font.cpal: Unsupported cpal table version."),
        }

        let color_records = Self::read_color_records(&mut stream, header.num_color_records);

        Ok(TableCpal {
            header,
            color_records,
        })
    }

    fn read_header(stream: &mut Stream, version: u16) -> Header {
        let num_palette_entries = stream.read_u16();
        let num_palettes = stream.read_u16();
        let num_color_records = stream.read_u16();
        let offset_first_color_record = stream.read_u32();
        stream.seek(offset_first_color_record as usize);
        let mut color_record_indicies = Vec::with_capacity(num_palettes as usize);
        for _ in 0..num_palettes {
            color_record_indicies.push(stream.read_u16());
        }

        // version 1 then has offset palette type array, offset palette label array, and offset palette entry label array,
        // the later two of which just provide UI names for colors and paletes, none of which matters for rasterizing
        // the first provides flags for weather a palette is usable on light and or dark backgrounds... which could be useful, maybe.
        Header {
            version,
            num_palette_entries,
            num_palettes,
            num_color_records,
            offset_first_color_record,
            color_record_indicies,
        }
    }

    fn read_color_records(stream: &mut Stream, num_color_records: u16) -> Vec<BGRA8Color> {
        let mut color_records = Vec::with_capacity(num_color_records as usize);
        for _ in 0..num_color_records {
            color_records.push(BGRA8Color {
                b: stream.read_u8(),
                g: stream.read_u8(),
                r: stream.read_u8(),
                a: stream.read_u8(),
            });
        }
        color_records
    }

    /// Gets the color at the given index from palette zero (which can be useful if you dont want to intelligently choose a palette)
    pub fn get_color(self: &Self, color_index: u16) -> BGRA8Color {
        self.get_color_from_palette(0, color_index)
    }

    /// Gets the color at a given index from a specific palette
    pub fn get_color_from_palette(self: &Self, palette: u16, color_index: u16) -> BGRA8Color {
        self.color_records[(self.header.color_record_indicies[palette as usize] + color_index) as usize]
    }
}
