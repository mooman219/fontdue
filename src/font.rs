use crate::math::*;
use crate::raster::Raster;
use crate::raw::RawFont;
use crate::FontResult;
use alloc::vec::*;
use core::ops::Deref;
use hashbrown::HashMap;

/// Axis aligned bounding box.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct AABB {
    /// Coordinate of the left-most edge.
    pub xmin: f32,
    /// Coordinate of the right-most edge.
    pub xmax: f32,
    /// Coordinate of the bottom-most edge.
    pub ymin: f32,
    /// Coordinate of the top-most edge.
    pub ymax: f32,
}

impl AABB {
    /// Creates a new axis aligned bounding box
    pub fn new(xmin: f32, xmax: f32, ymin: f32, ymax: f32) -> AABB {
        AABB {
            xmin,
            xmax,
            ymin,
            ymax,
        }
    }

    /// Scales the bounding box by the given factor.
    #[inline(always)]
    pub fn scale(&self, scale: f32) -> AABB {
        AABB {
            xmin: self.xmin * scale,
            xmax: self.xmax * scale,
            ymin: self.ymin * scale,
            ymax: self.ymax * scale,
        }
    }
}

/// Encapsulates all layout information associated with a glyph for a fixed scale.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Metrics {
    /// The width of the associated glyph in pixels.
    pub width: usize,
    /// The height of the associated glyph in pixels.
    pub height: usize,
    /// Advance width of the glyph. Used in horizontal fonts.
    pub advance_width: f32,
    /// Advance height of the glyph. Used in vertical fonts.
    pub advance_height: f32,
    /// Inner bounds of the glyph at the offsets specified by the font.
    pub bounds: AABB,
}

struct Glyph {
    geometry: Geometry,
    width: f32,
    height: f32,
    advance_width: f32,
    advance_height: f32,
    bounds: AABB,
}

impl Glyph {
    #[inline(always)]
    fn metrics(&self, scale: f32) -> Metrics {
        Metrics {
            width: (scale * self.width).ceil() as usize,
            height: (scale * self.height).ceil() as usize,
            advance_width: scale * self.advance_width,
            advance_height: scale * self.advance_height,
            bounds: self.bounds.scale(scale),
        }
    }
}

/// Settings for controling specific font and layout behavior.
#[derive(Copy, Clone, PartialEq)]
pub struct FontSettings {}

impl Default for FontSettings {
    fn default() -> FontSettings {
        FontSettings {}
    }
}

/// Represents a font. Fonts are immutable after creation and owns its own copy of the font data.
pub struct Font {
    units_per_em: f32,
    glyphs: Vec<Glyph>,
    char_to_glyph: HashMap<u32, u32>,
    // Metrics
    new_line_width: f32,
    new_line_height: f32,
    has_horizontal_metrics: bool,
    has_vertical_metrics: bool,
}

impl Font {
    /// Constructs a font from an array of bytes.
    pub fn from_bytes<Data: Deref<Target = [u8]>>(data: Data, _: FontSettings) -> FontResult<Font> {
        let mut raw = RawFont::new(data)?;

        let mut glyphs = Vec::with_capacity(raw.glyf.glyphs.len());
        for glyph in &mut raw.glyf.glyphs {
            // Invert and offset the geometry here.
            glyph.reposition();
            let geometry = compile(&glyph.points);
            // Glyph metrics.
            let advance_width = if let Some(hmtx) = &raw.hmtx {
                let hmetric = hmtx.hmetrics[glyph.metrics];
                hmetric.advance_width as f32
            } else {
                0.0
            };
            let advance_height = if let Some(vmtx) = &raw.vmtx {
                let vmetric = vmtx.vmetrics[glyph.metrics];
                vmetric.advance_height as f32
            } else {
                0.0
            };
            let bounds =
                AABB::new(glyph.xmin as f32, glyph.xmax as f32, glyph.ymin as f32, glyph.ymax as f32);
            // Construct the glyph.
            glyphs.push(Glyph {
                geometry,
                width: (glyph.xmax - glyph.xmin) as f32,
                height: (glyph.ymax - glyph.ymin) as f32,
                advance_width,
                advance_height,
                bounds,
            });
        }

        // New line metrics.
        let (has_horizontal_metrics, new_line_height) = if let Some(hhea) = &raw.hhea {
            (true, (hhea.ascent - hhea.descent + hhea.line_gap) as f32)
        } else {
            (false, 0.0)
        };
        let (has_vertical_metrics, new_line_width) = if let Some(vhea) = &raw.vhea {
            (true, (vhea.ascent - vhea.descent + vhea.line_gap) as f32)
        } else {
            (false, 0.0)
        };

        Ok(Font {
            glyphs,
            char_to_glyph: raw.cmap.map.clone(),
            units_per_em: raw.head.units_per_em as f32,
            new_line_height,
            new_line_width,
            has_horizontal_metrics,
            has_vertical_metrics,
        })
    }

    /// The new line height for the font. Only populated for fonts with vertical text layout
    /// metrics. Zero if unpopulated.
    pub fn new_line_width(&self) -> f32 {
        self.new_line_width
    }

    /// The new line height for the font. Only populated for fonts with horizontal text layout
    /// metrics. Zero if unpopulated.
    pub fn new_line_height(&self) -> f32 {
        self.new_line_height
    }

    /// Returns true if the font provides horizontal text layout metrics.
    pub fn has_horizontal_metrics(&self) -> bool {
        self.has_horizontal_metrics
    }

    /// Returns true if the font provides vertical text layout metrics.
    pub fn has_vertical_metrics(&self) -> bool {
        self.has_vertical_metrics
    }

    /// Calculates the glyph scale factor for a given px size.
    #[inline(always)]
    fn scale_factor(px: f32, units_per_em: f32) -> f32 {
        px / units_per_em
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
        let scale = Font::scale_factor(px, self.units_per_em);
        glyph.metrics(scale)
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
        // TODO: Fix rounding errors at high scalings.
        assert!(px <= 5000.0);
        let glyph = &self.glyphs[index];
        let scale = Font::scale_factor(px, self.units_per_em);
        let metrics = glyph.metrics(scale);
        let mut canvas = Raster::new(metrics.width, metrics.height);
        canvas.draw(&glyph.geometry, scale);
        (metrics, canvas.get_bitmap())
    }

    /// Finds the internal glyph index for the given character. If the character is not present in
    /// the font then 0 is returned.
    pub fn lookup_glyph_index(&self, character: char) -> usize {
        let result = self.char_to_glyph.get(&(character as u32));
        match result.copied() {
            Some(index) => index as usize,
            None => 0,
        }
    }
}
