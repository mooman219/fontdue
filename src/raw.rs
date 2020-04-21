pub use crate::table::*;
use crate::FontResult;
use core::ops::Deref;

pub struct RawFont {
    pub head: TableHead,
    pub cmap: TableCmap,
    pub maxp: TableMaxp,
    pub loca: TableLoca,
    pub glyf: TableGlyf,
    pub kern: Option<TableKern>,

    pub hhea: Option<TableHhea>,
    pub hmtx: Option<TableHmtx>,

    pub vhea: Option<TableVhea>,
    pub vmtx: Option<TableVmtx>,

    pub cpal: Option<TableCpal>,
    pub colr: Option<TableColr>,
}

impl RawFont {
    pub fn new<Data: Deref<Target = [u8]>>(data: Data) -> FontResult<RawFont> {
        let dir = TableDirectory::new(&data)?;

        // Font infromation (Required)
        let head_offset = dir.map.get(b"head").expect("Font: Missing head table").offset;
        let maxp_offset = dir.map.get(b"maxp").expect("Font: Missing maxp table").offset;
        let head = TableHead::new(&data[head_offset..])?;
        let maxp = TableMaxp::new(&data[maxp_offset..])?;

        // Character mapping (Required)
        let cmap_offset = dir.map.get(b"cmap").expect("Font: Missing cmap table").offset;
        let cmap = TableCmap::new(&data[cmap_offset..])?;

        // Glyph outline information (Required)
        let loca_offset = dir.map.get(b"loca").expect("Font: Missing loca table").offset;
        let glyf_offset = dir.map.get(b"glyf").expect("Font: Missing glyf table").offset;
        let loca = TableLoca::new(&data[loca_offset..], head.index_to_loc_format, maxp.num_glyphs)?;
        let glyf = TableGlyf::new(&data[glyf_offset..], &loca.locations)?;

        // Kerning
        let kern_offset = dir.map.get(b"kern").map(|v| v.offset);
        let kern = if let Some(kern_offset) = kern_offset {
            let kern = TableKern::new(&data[kern_offset..])?;
            Some(kern)
        } else {
            None
        };

        // Horizontal information (Optional)
        let hhea_offset = dir.map.get(b"hhea").map(|v| v.offset);
        let (hhea, hmtx) = if let Some(hhea_offset) = hhea_offset {
            let hmtx_offset = dir.map.get(b"hmtx").expect("Font: Found hhea, missing hmtx table").offset;
            let hhea = TableHhea::new(&data[hhea_offset..])?;
            let hmtx = TableHmtx::new(&data[hmtx_offset..], maxp.num_glyphs, hhea.num_long_hmetrics)?;
            (Some(hhea), Some(hmtx))
        } else {
            (None, None)
        };

        // Vertical information (Optional)
        let vhea_offset = dir.map.get(b"vhea").map(|v| v.offset);
        let (vhea, vmtx) = if let Some(vhea_offset) = vhea_offset {
            let vmtx_offset = dir.map.get(b"vmtx").expect("Font: Found vhea, missing vmtx table").offset;
            let vhea = TableVhea::new(&data[vhea_offset..])?;
            let vmtx = TableVmtx::new(&data[vmtx_offset..], maxp.num_glyphs, vhea.num_long_vmetrics)?;
            (Some(vhea), Some(vmtx))
        } else {
            (None, None)
        };

        // Color pallete
        let cpal_offset = dir.map.get(b"CPAL").map(|v| v.offset);
        let cpal = if let Some(cpal_offset) = cpal_offset {
            let cpal = TableCpal::new(&data[cpal_offset..])?;
            Some(cpal)
        } else {
            None
        };

        // Color layers
        let colr_offset = dir.map.get(b"COLR").map(|v| v.offset);
        let colr = if let Some(colr_offset) = colr_offset {
            if cpal.is_none() {
                panic!("Font: found COLR, missing CPAL table");
            }
            let colr = TableColr::new(&data[colr_offset..])?;
            Some(colr)
        } else {
            None
        };

        Ok(RawFont {
            head,
            cmap,
            maxp,
            loca,
            hhea,
            hmtx,
            glyf,
            kern,
            vhea,
            vmtx,
            cpal,
            colr,
        })
    }
}
