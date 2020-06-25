use crate::layout::GlyphRasterConfig;
use crate::math;
use crate::math::Geometry;
use crate::raster::Raster;
use crate::raw::RawFont;
use crate::simd::ceil;
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

/// Metrics associated with line positioning.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct LineMetrics {
    /// The highest point that any glyph in the font extends to above the baseline. Typically
    /// positive.
    pub ascent: f32,
    /// The lowest point that any glyph in the font extends to below the baseline. Typically
    /// negative.
    pub descent: f32,
    /// The gap to leave between the descent of one line and the ascent of the next. This is of
    /// course only a guideline given by the font's designers.
    pub line_gap: f32,
    /// A precalculated value for the height or width of the line depending on if the font is laid
    /// out horizontally or vertically. It's calculated by: ascent - descent + line_gap.
    pub new_line_size: f32,
}

impl LineMetrics {
    /// Creates a new line metrics struct and computes the new line size.
    pub fn new(ascent: i16, descent: i16, line_gap: i16) -> LineMetrics {
        LineMetrics {
            ascent: ascent as f32,
            descent: descent as f32,
            line_gap: line_gap as f32,
            new_line_size: (ascent - descent + line_gap) as f32,
        }
    }

    /// Scales the line metrics by the given factor.
    #[inline(always)]
    pub fn scale(&self, scale: f32) -> LineMetrics {
        LineMetrics {
            ascent: self.ascent * scale,
            descent: self.descent * scale,
            line_gap: self.line_gap * scale,
            new_line_size: self.new_line_size * scale,
        }
    }
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
    fn metrics(&self, scale: f32, offset: f32) -> Metrics {
        Metrics {
            width: ceil(scale * self.width + offset) as usize,
            height: ceil(scale * self.height) as usize,
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
    horizontal_line_metrics: Option<LineMetrics>,
    vertical_line_metrics: Option<LineMetrics>,
}

impl Font {
    /// Constructs a font from an array of bytes.
    pub fn from_bytes<Data: Deref<Target = [u8]>>(data: Data, _: FontSettings) -> FontResult<Font> {
        let mut raw = RawFont::new(data)?;

        let mut glyphs = Vec::with_capacity(raw.glyf.glyphs.len());
        for glyph in &mut raw.glyf.glyphs {
            // Invert and offset the geometry here.
            glyph.reposition();
            let geometry = math::compile(&glyph.points);
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
            glyphs.push(Glyph {
                geometry,
                width: bounds.xmax - bounds.xmin,
                height: bounds.ymax - bounds.ymin,
                advance_width,
                advance_height,
                bounds,
            });
        }

        // New line metrics.
        let horizontal_line_metrics = if let Some(hhea) = &raw.hhea {
            Some(LineMetrics::new(hhea.ascent, hhea.descent, hhea.line_gap))
        } else {
            None
        };
        let vertical_line_metrics = if let Some(vhea) = &raw.vhea {
            Some(LineMetrics::new(vhea.ascent, vhea.descent, vhea.line_gap))
        } else {
            None
        };

        Ok(Font {
            glyphs,
            char_to_glyph: raw.cmap.map.clone(),
            units_per_em: raw.head.units_per_em as f32,
            horizontal_line_metrics,
            vertical_line_metrics,
        })
    }

    /// The new line height for the font. Only populated for fonts with vertical text layout
    /// metrics. None if unpopulated.
    pub fn horizontal_line_metrics(&self, px: f32) -> Option<LineMetrics> {
        if let Some(metrics) = self.horizontal_line_metrics {
            Some(metrics.scale(Self::scale_factor(px, self.units_per_em)))
        } else {
            None
        }
    }

    /// The new line height for the font. Only populated for fonts with horizontal text layout
    /// metrics. None if unpopulated.
    pub fn vertical_line_metrics(&self, px: f32) -> Option<LineMetrics> {
        if let Some(metrics) = self.vertical_line_metrics {
            Some(metrics.scale(Self::scale_factor(px, self.units_per_em)))
        } else {
            None
        }
    }

    /// Calculates the glyph scale factor for a given px size.
    #[inline(always)]
    fn scale_factor(px: f32, units_per_em: f32) -> f32 {
        px / units_per_em
    }

    /// Retrieves the layout metrics for the given character. If the caracter isn't present in the
    /// font, then the layout for the font's default character is returned instead.
    /// # Arguments
    ///
    /// * `index` - The character in the font to to generate the layout metrics for.
    /// * `px` - The size to generate the layout metrics for the character at.
    /// * `offset` - The horizontal offset to generate the layout metrics for the character at. An
    /// offset of 0.0 applies no offsetting.
    pub fn metrics(&self, character: char, px: f32, offset: f32) -> Metrics {
        self.metrics_indexed(self.lookup_glyph_index(character), px, offset)
    }

    /// Retrieves the layout metrics at the given index. You normally want to be using
    /// metrics(char, f32) instead, unless your glyphs are pre-indexed.
    /// # Arguments
    ///
    /// * `index` - The glyph index in the font to to generate the layout metrics for.
    /// * `px` - The size to generate the layout metrics for the glyph at.
    /// * `offset` - The horizontal offset to generate the layout metrics for the glyph at. An
    /// offset of 0.0 applies no offsetting.
    pub fn metrics_indexed(&self, index: usize, px: f32, offset: f32) -> Metrics {
        let glyph = &self.glyphs[index];
        let scale = Font::scale_factor(px, self.units_per_em);
        glyph.metrics(scale, offset)
    }

    /// Retrieves the layout metrics and rasterized bitmap for the given character. If the caracter
    /// isn't present in the font, then the layout and bitmap for the font's default character is
    /// returned instead.
    /// # Arguments
    ///
    /// * `character` - The character to rasterize.
    /// * `px` - The size to render the character at.
    /// * `offset` - The horizontal offset to render the character at. An offset of 0.0 applies no
    /// offsetting.
    pub fn rasterize_config(&self, config: GlyphRasterConfig) -> (Metrics, Vec<u8>) {
        self.rasterize_indexed(self.lookup_glyph_index(config.c), config.px, config.offset as f32 / 256.0)
    }

    /// Retrieves the layout metrics and rasterized bitmap for the given character. If the caracter
    /// isn't present in the font, then the layout and bitmap for the font's default character is
    /// returned instead.
    /// # Arguments
    ///
    /// * `character` - The character to rasterize.
    /// * `px` - The size to render the character at.
    /// * `offset` - The horizontal offset to render the character at. An offset of 0.0 applies no
    /// offsetting.
    pub fn rasterize(&self, character: char, px: f32, offset: f32) -> (Metrics, Vec<u8>) {
        self.rasterize_indexed(self.lookup_glyph_index(character), px, offset)
    }

    /// Retrieves the layout metrics and rasterized bitmap at the given index. You normally want to
    /// be using rasterize(char, f32) instead, unless your glyphs are pre-indexed.
    /// # Arguments
    ///
    /// * `index` - The glyph index in the font to rasterize.
    /// * `px` - The size to render the character at.
    /// * `offset` - The horizontal offset to render the glyph at. An offset of 0.0 applies no
    /// offsetting.
    pub fn rasterize_indexed(&self, index: usize, px: f32, offset: f32) -> (Metrics, Vec<u8>) {
        let glyph = &self.glyphs[index];
        let scale = Font::scale_factor(px, self.units_per_em);
        let metrics = glyph.metrics(scale, offset);
        let mut canvas = Raster::new(metrics.width, metrics.height);
        canvas.draw(&glyph.geometry, scale, offset);
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
