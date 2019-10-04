use crate::math::*;
use crate::raster::Raster;
use crate::raw::RawFont;
use crate::FontResult;
use alloc::vec::*;
use core::ops::Deref;
use hashbrown::HashMap;

/// Encapsulates all layout information associated with a glyph for a fixed scale.
#[derive(Copy, Clone, PartialEq)]
pub struct Metrics {
    /// The scale used to adjust the glyph points by.
    scale: f32,
    /// The width of the associated glyph.
    pub width: usize,
    /// The height of the associated glyph.
    pub height: usize,
    /// The left side bearing. Used in horizontal fonts.
    pub bearing_x: f32,
    /// The top side bearing. Used in vertical fonts.
    pub bearing_y: f32,
    /// The advance width. Used in horizontal fonts.
    pub advance_x: f32,
    /// The advance height. Used in vertical fonts.
    pub advance_y: f32,
}

struct Glyph {
    geometry: Vec<Geometry>,
    width: f32,
    height: f32,
}

impl Glyph {
    fn metrics(&self, px: f32, units_per_em: f32) -> Metrics {
        let scale = px / units_per_em;
        Metrics {
            scale,
            width: (scale * self.width).ceil() as usize,
            height: (scale * self.height).ceil() as usize,
            bearing_x: 0.0,
            bearing_y: 0.0,
            advance_x: 0.0,
            advance_y: 0.0,
        }
    }
}

pub struct FontSettings {
    /// Transforms all glyphs to be flipped vertically. False by default.
    pub flip_vertical: bool,
}

impl Default for FontSettings {
    fn default() -> FontSettings {
        FontSettings {
            flip_vertical: false,
        }
    }
}

/// Represents a font. Fonts are immutable after creation and owns its own copy of the font data.
pub struct Font {
    units_per_em: f32,
    glyphs: Vec<Glyph>,
    char_to_glyph: HashMap<u32, u32>,
}

impl Font {
    /// Constructs a font from an array of bytes.
    pub fn from_bytes<Data: Deref<Target = [u8]>>(data: Data, settings: FontSettings) -> FontResult<Font> {
        let raw = RawFont::new(data)?;

        let mut glyphs = Vec::with_capacity(raw.glyf.glyphs.len());
        for glyph in &raw.glyf.glyphs {
            // Invert and offset the geometry here.
            let mut geometry = to_geometry(&glyph.points);
            for element in &mut geometry {
                *element = element.offset(-glyph.xmin as f32, -glyph.ymin as f32);
                if settings.flip_vertical {
                    *element = element.mirror_x((glyph.ymax - glyph.ymin) as f32 / 2.0);
                }
            }
            // Construct the glyph.
            glyphs.push(Glyph {
                geometry,
                width: (glyph.xmax - glyph.xmin) as f32,
                height: (glyph.ymax - glyph.ymin) as f32,
            });
        }

        Ok(Font {
            glyphs,
            char_to_glyph: raw.cmap.map.clone(),
            units_per_em: raw.head.units_per_em as f32,
        })
    }

    /// Retrieves the layout metrics for the given character. If the caracter isn't present in the
    /// font, then the layout for the font's default character is returned instead.
    pub fn metrics(&self, character: char, px: f32) -> Metrics {
        self.metrics_indexed(self.lookup_glyph_index(character), px)
    }

    /// Retrieves the layout metrics at the given index. You normally want to be using
    /// metrics(char, f32) instead, unless your glyphs are pre-indexed.
    pub fn metrics_indexed(&self, index: usize, px: f32) -> Metrics {
        let glyph = &self.glyphs[index];
        glyph.metrics(px, self.units_per_em)
    }

    /// Retrieves the layout metrics and rasterized bitmap for the given character. If the caracter
    /// isn't present in the font, then the layout and bitmap for the font's default character is
    /// returned instead.
    pub fn rasterize(&self, character: char, px: f32) -> (Metrics, Vec<u8>) {
        self.rasterize_indexed(self.lookup_glyph_index(character), px)
    }

    /// Retrieves the layout metrics and rasterized bitmap at the given index. You normally want to
    /// be using rasterize(char, f32) instead, unless your glyphs are pre-indexed.
    pub fn rasterize_indexed(&self, index: usize, px: f32) -> (Metrics, Vec<u8>) {
        let glyph = &self.glyphs[index];
        let metrics = glyph.metrics(px, self.units_per_em);
        let mut canvas = Raster::new(metrics.width, metrics.height);
        for element in &glyph.geometry {
            canvas.draw(&element.scale(metrics.scale));
        }
        (metrics, canvas.get_bitmap())
    }

    pub fn lookup_glyph_index(&self, character: char) -> usize {
        let result = self.char_to_glyph.get(&(character as u32));
        match result.copied() {
            Some(index) => index as usize,
            None => 0,
        }
    }
}
