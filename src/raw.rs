use crate::parse::*;
pub use crate::table::*;
use crate::FontResult;
use core::ops::Deref;

// Apple: https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6.html
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/otff

fn table_offset(data: &[u8], fontstart: usize, tag: &[u8]) -> Option<usize> {
    let num_tables = read_u16(&data[fontstart + 4..]);
    let tabledir = fontstart + 12;
    for i in 0..num_tables {
        let loc = tabledir + 16 * (i as usize);
        if &data[loc..loc + 4] == tag {
            return Some(read_u32(&data[loc + 8..]) as usize);
        }
    }
    None
}

fn is_font(data: &[u8]) -> bool {
    if data.len() >= 4 {
        let tag = read_u32(data);
        tag == 0x7472_7565 // true
        || tag == 0x7479_7031 // typ1
        || tag == 0x4F54_544F // OTTO
        || tag == 0x0001_0000 // The normal one
    } else {
        false
    }
}

fn is_collection(data: &[u8]) -> bool {
    if data.len() >= 4 {
        let tag = read_u32(data);
        tag == 0x7474_6366 // ttcf
    } else {
        false
    }
}

pub struct RawFont {
    pub head: TableHead,
    pub cmap: TableCmap,
    pub maxp: TableMaxp,
    pub loca: TableLoca,
    pub glyf: TableGlyf,

    pub hhea: Option<TableHhea>,
    pub hmtx: Option<TableHmtx>,

    pub vhea: Option<TableVhea>,
    pub vmtx: Option<TableVmtx>,
}

impl RawFont {
    pub fn new<Data: Deref<Target = [u8]>>(data: Data) -> FontResult<RawFont> {
        if !is_font(&data) {
            return Err("Font: This is not a parsable font.");
        }
        // Font infromation (Required)
        let head_offset = table_offset(&data, 0, b"head").expect("Font: Missing head table");
        let maxp_offset = table_offset(&data, 0, b"maxp").expect("Font: Missing maxp table");
        let head = TableHead::new(&data[head_offset..])?;
        let maxp = TableMaxp::new(&data[maxp_offset..])?;

        // Character mapping (Required)
        let cmap_offset = table_offset(&data, 0, b"cmap").expect("Font: Missing cmap table");
        let cmap = TableCmap::new(&data[cmap_offset..])?;

        // Glyph outline information (Required)
        let loca_offset = table_offset(&data, 0, b"loca").expect("Font: Missing loca table");
        let glyf_offset = table_offset(&data, 0, b"glyf").expect("Font: Missing glyf table");
        let loca = TableLoca::new(&data[loca_offset..], head.index_to_loc_format, maxp.num_glyphs)?;
        let glyf = TableGlyf::new(&data[glyf_offset..], &loca.locations)?;

        // Horizontal information (Optional)
        let hhea_offset = table_offset(&data, 0, b"hhea");
        let (hhea, hmtx) = if let Some(hhea_offset) = hhea_offset {
            let hmtx_offset = table_offset(&data, 0, b"hmtx").expect("Font: Found hhea, missing hmtx table");
            let hhea = TableHhea::new(&data[hhea_offset..])?;
            let hmtx = TableHmtx::new(&data[hmtx_offset..], maxp.num_glyphs, hhea.num_long_hmetrics)?;
            (Some(hhea), Some(hmtx))
        } else {
            (None, None)
        };

        // Vertical information (Optional)
        let vhea_offset = table_offset(&data, 0, b"vhea");
        let (vhea, vmtx) = if let Some(vhea_offset) = vhea_offset {
            let vmtx_offset = table_offset(&data, 0, b"vmtx").expect("Font: Found vhea, missing vmtx table");
            let vhea = TableVhea::new(&data[vhea_offset..])?;
            let vmtx = TableVmtx::new(&data[vmtx_offset..], maxp.num_glyphs, vhea.num_long_vmetrics)?;
            (Some(vhea), Some(vmtx))
        } else {
            (None, None)
        };

        Ok(RawFont {
            head,
            cmap,
            maxp,
            loca,
            hhea,
            hmtx,
            glyf,
            vhea,
            vmtx,
        })
    }
}
