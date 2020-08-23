use crate::layout::GlyphRasterConfig;
use crate::math::Geometry;
use crate::platform::{as_i32, ceil, fract, is_negative};
use crate::raster::Raster;
use crate::raw::RawFont;
use crate::FontResult;
use alloc::vec::*;
use core::mem;
use core::num::NonZeroU32;
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

/// Metric representing a glyph that has no geometry and positional metrics.
pub const ZERO_METRICS: Metrics = Metrics {
    width: 0,
    height: 0,
    advance_width: 0.0,
    advance_height: 0.0,
    bounds: AABB {
        xmin: 0.0,
        xmax: 0.0,
        ymin: 0.0,
        ymax: 0.0,
    },
};

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
    fn metrics(&self, scale: f32, enable_offset_bounding_box: bool) -> Metrics {
        let bounds = self.bounds.scale(scale);
        let height = scale * self.height;
        let (offset_x, offset_y) = if enable_offset_bounding_box {
            let mut offset_x = fract(bounds.xmin);
            if is_negative(offset_x) {
                offset_x += 1.0;
            }
            let mut offset_y = ceil(height) - height - fract(bounds.ymin);
            if is_negative(offset_y) {
                offset_y += 1.0;
            }
            (offset_x, offset_y)
        } else {
            (0.0, 0.0)
        };
        Metrics {
            width: as_i32(ceil(scale * self.width + offset_x)) as usize,
            height: as_i32(ceil(height + offset_y)) as usize,
            advance_width: scale * self.advance_width,
            advance_height: scale * self.advance_height,
            bounds,
        }
    }
}

/// Settings for controlling specific font and layout behavior.
#[derive(Copy, Clone, PartialEq)]
pub struct FontSettings {
    /// The default is 0.0. Offsets all glyphs in the font horizontally by the given offset. The
    /// unit of the offset is font units and varies by font. This can be used to make a font
    /// crisper at a specific scale, or correct a misaligned font.
    pub offset_x: f32,
    /// The default is 0.0. Offsets all glyphs in the font vertically by the given offset. The unit
    /// of the offset is font units and varies by font. This can be used to make a font crisper at
    /// a specific scale, or correct a misaligned font.
    pub offset_y: f32,
    /// The default is true. This offsets glyphs relative to their position in their scaled
    /// bounding box. This is required for laying out glyphs correctly, but can be disabled to make
    /// some incorrect fonts crisper.
    pub enable_offset_bounding_box: bool,
}

impl Default for FontSettings {
    fn default() -> FontSettings {
        FontSettings {
            offset_x: 0.0,
            offset_y: 0.0,
            enable_offset_bounding_box: true,
        }
    }
}

/// Represents a font. Fonts are immutable after creation and owns its own copy of the font data.
pub struct Font {
    units_per_em: f32,
    glyphs: Vec<Glyph>,
    char_to_glyph: HashMap<u32, NonZeroU32>,
    horizontal_line_metrics: Option<LineMetrics>,
    vertical_line_metrics: Option<LineMetrics>,
    settings: FontSettings,
}

impl Font {
    /// Constructs a font from an array of bytes.
    pub fn from_bytes<Data: Deref<Target = [u8]>>(data: Data, settings: FontSettings) -> FontResult<Font> {
        let mut raw = RawFont::new(data)?;

        let mut glyphs = Vec::with_capacity(raw.glyf.glyphs.len());
        for glyph in &mut raw.glyf.glyphs {
            let geometry = Geometry::new(&glyph.points, &settings);
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
            let bounds = geometry.effective_bounds.unwrap_or(AABB::new(0.0, 0.0, 0.0, 0.0));
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
            settings,
        })
    }

    /// The new line height for the font. Only populated for fonts with vertical text layout
    /// metrics. None if unpopulated.
    pub fn horizontal_line_metrics(&self, px: f32) -> Option<LineMetrics> {
        if let Some(metrics) = self.horizontal_line_metrics {
            Some(metrics.scale(self.scale_factor(px)))
        } else {
            None
        }
    }

    /// The new line height for the font. Only populated for fonts with horizontal text layout
    /// metrics. None if unpopulated.
    pub fn vertical_line_metrics(&self, px: f32) -> Option<LineMetrics> {
        if let Some(metrics) = self.vertical_line_metrics {
            Some(metrics.scale(self.scale_factor(px)))
        } else {
            None
        }
    }

    /// Calculates the glyph scale factor for a given px size.
    #[inline(always)]
    fn scale_factor(&self, px: f32) -> f32 {
        px / self.units_per_em
    }

    /// Retrieves the layout metrics for the given character. If the character isn't present in the
    /// font, then the layout for the font's default character is returned instead.
    /// # Arguments
    ///
    /// * `index` - The character in the font to to generate the layout metrics for.
    /// * `px` - The size to generate the layout metrics for the character at. Cannot be negative.
    /// # Returns
    ///
    /// * `Metrics` - Sizing and positioning metadata for the glyph.
    #[inline]
    pub fn metrics(&self, character: char, px: f32) -> Metrics {
        self.metrics_indexed(self.lookup_glyph_index(character), px)
    }

    /// Retrieves the layout metrics at the given index. You normally want to be using
    /// metrics(char, f32) instead, unless your glyphs are pre-indexed.
    /// # Arguments
    ///
    /// * `index` - The glyph index in the font to to generate the layout metrics for.
    /// * `px` - The size to generate the layout metrics for the glyph at. Cannot be negative.
    /// # Returns
    ///
    /// * `Metrics` - Sizing and positioning metadata for the glyph.
    pub fn metrics_indexed(&self, index: usize, px: f32) -> Metrics {
        let glyph = &self.glyphs[index];
        let scale = self.scale_factor(px);
        glyph.metrics(scale, self.settings.enable_offset_bounding_box)
    }

    /// Retrieves the layout rasterized bitmap for the given raster config. If the raster config's
    /// character isn't present in the font, then the layout and bitmap for the font's default
    /// character's raster is returned instead.
    /// # Arguments
    ///
    /// * `config` - The settings to render the character at.
    /// # Returns
    ///
    /// * `Metrics` - Sizing and positioning metadata for the rasterized glyph.
    /// * `Vec<u8>` - Coverage vector for the glyph. Coverage is a linear scale where 0 represents
    /// 0% coverage of that pixel by the glyph and 255 represents 100% coverage. The vec starts at
    /// the top left corner of the glyph.
    #[inline]
    pub fn rasterize_config(&self, config: GlyphRasterConfig) -> (Metrics, Vec<u8>) {
        self.rasterize_indexed(self.lookup_glyph_index(config.c), config.px)
    }

    /// Retrieves the layout metrics and rasterized bitmap for the given character. If the
    /// character isn't present in the font, then the layout and bitmap for the font's default
    /// character is returned instead.
    /// # Arguments
    ///
    /// * `character` - The character to rasterize.
    /// * `px` - The size to render the character at. Cannot be negative.
    /// # Returns
    ///
    /// * `Metrics` - Sizing and positioning metadata for the rasterized glyph.
    /// * `Vec<u8>` - Coverage vector for the glyph. Coverage is a linear scale where 0 represents
    /// 0% coverage of that pixel by the glyph and 255 represents 100% coverage. The vec starts at
    /// the top left corner of the glyph.
    #[inline]
    pub fn rasterize(&self, character: char, px: f32) -> (Metrics, Vec<u8>) {
        self.rasterize_indexed(self.lookup_glyph_index(character), px)
    }

    /// Retrieves the layout metrics and rasterized bitmap at the given index. You normally want to
    /// be using rasterize(char, f32) instead, unless your glyphs are pre-indexed.
    /// # Arguments
    ///
    /// * `index` - The glyph index in the font to rasterize.
    /// * `px` - The size to render the character at. Cannot be negative.
    /// # Returns
    ///
    /// * `Metrics` - Sizing and positioning metadata for the rasterized glyph.
    /// * `Vec<u8>` - Coverage vector for the glyph. Coverage is a linear scale where 0 represents
    /// 0% coverage of that pixel by the glyph and 255 represents 100% coverage. The vec starts at
    /// the top left corner of the glyph.
    pub fn rasterize_indexed(&self, index: usize, px: f32) -> (Metrics, Vec<u8>) {
        let glyph = &self.glyphs[index];
        let scale = self.scale_factor(px);

        // This is kinda lame, but I can't reuse Glyph.metrics() directly because I want the
        // offset_x and offset_y too, and returning it gave a weird regression.
        let bounds = glyph.bounds.scale(scale);
        let height = scale * glyph.height;
        let (offset_x, offset_y) = if self.settings.enable_offset_bounding_box {
            let mut offset_x = fract(bounds.xmin);
            if is_negative(offset_x) {
                offset_x += 1.0;
            }
            let mut offset_y = ceil(height) - height - fract(bounds.ymin);
            if is_negative(offset_y) {
                offset_y += 1.0;
            }
            (offset_x, offset_y)
        } else {
            (0.0, 0.0)
        };
        let metrics = Metrics {
            width: as_i32(ceil(scale * glyph.width + offset_x)) as usize,
            height: as_i32(ceil(height + offset_y)) as usize,
            advance_width: scale * glyph.advance_width,
            advance_height: scale * glyph.advance_height,
            bounds,
        };

        let mut canvas = Raster::new(metrics.width, metrics.height);
        canvas.draw(&glyph.geometry, scale, offset_x, offset_y);
        (metrics, canvas.get_bitmap())
    }

    /// Finds the internal glyph index for the given character. If the character is not present in
    /// the font then 0 is returned.
    #[inline]
    pub fn lookup_glyph_index(&self, character: char) -> usize {
        unsafe {
            mem::transmute::<Option<NonZeroU32>, u32>(self.char_to_glyph.get(&(character as u32)).copied())
                as usize
        }
    }
}
