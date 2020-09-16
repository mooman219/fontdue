use crate::layout::GlyphRasterConfig;
use crate::math::{Geometry, Line};
use crate::platform::{as_i32, ceil, fract, is_negative};
use crate::raster::Raster;
use crate::FontResult;
use alloc::vec;
use alloc::vec::*;
use core::mem;
use core::num::NonZeroU16;
use core::ops::Deref;
use hashbrown::HashMap;
use ttf_parser::FaceParsingError;

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

impl Default for AABB {
    fn default() -> Self {
        AABB {
            xmin: 0.0,
            xmax: 0.0,
            ymin: 0.0,
            ymax: 0.0,
        }
    }
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
    /// The width of the associated glyph in whole pixels.
    pub width: usize,
    /// The height of the associated glyph in whole pixels.
    pub height: usize,
    /// Advance width of the glyph. Used in horizontal fonts.
    pub advance_width: f32,
    /// Advance height of the glyph. Used in vertical fonts.
    pub advance_height: f32,
    /// Inner bounds of the glyph at the offsets specified by the font.
    pub bounds: AABB,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
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
        }
    }
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
    fn new(ascent: i16, descent: i16, line_gap: i16) -> LineMetrics {
        // Operations between this values can exceed i16, so we extend to i32 here.
        let (ascent, descent, line_gap) = (ascent as i32, descent as i32, line_gap as i32);
        LineMetrics {
            ascent: ascent as f32,
            descent: descent as f32,
            line_gap: line_gap as f32,
            new_line_size: (ascent - descent + line_gap) as f32,
        }
    }

    /// Scales the line metrics by the given factor.
    #[inline(always)]
    fn scale(&self, scale: f32) -> LineMetrics {
        LineMetrics {
            ascent: self.ascent * scale,
            descent: self.descent * scale,
            line_gap: self.line_gap * scale,
            new_line_size: self.new_line_size * scale,
        }
    }
}

#[derive(Clone)]
pub(crate) struct Glyph {
    pub v_lines: Vec<Line>,
    pub m_lines: Vec<Line>,
    width: f32,
    height: f32,
    advance_width: f32,
    advance_height: f32,
    bounds: AABB,
}

impl Default for Glyph {
    fn default() -> Self {
        Glyph {
            v_lines: Vec::new(),
            m_lines: Vec::new(),
            width: 0.0,
            height: 0.0,
            advance_width: 0.0,
            advance_height: 0.0,
            bounds: AABB::default(),
        }
    }
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
            let mut offset_y = fract(1.0 - fract(height) - fract(bounds.ymin));
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
    /// The default is true. This offsets glyphs relative to their position in their scaled
    /// bounding box. This is required for laying out glyphs correctly, but can be disabled to make
    /// some incorrect fonts crisper.
    pub enable_offset_bounding_box: bool,
    /// The default is 0. The index of the font to use if parsing a font collection.
    pub collection_index: u32,
    /// The default is 40. The scale in px the font geometry is optimized for. Fonts rendered at
    /// the scale defined here will be the most optimal in terms of looks and performance. Glyphs
    /// rendered smaller than this scale will look the same but perform slightly worse, while
    /// glyphs rendered larger than this will looks worse but perform slightly better.
    pub scale: f32,
}

impl Default for FontSettings {
    fn default() -> FontSettings {
        FontSettings {
            enable_offset_bounding_box: true,
            collection_index: 0,
            scale: 40.0,
        }
    }
}

/// Represents a font. Fonts are immutable after creation and owns its own copy of the font data.
pub struct Font {
    units_per_em: f32,
    glyphs: Vec<Glyph>,
    char_to_glyph: HashMap<u32, NonZeroU16>,
    horizontal_line_metrics: Option<LineMetrics>,
    vertical_line_metrics: Option<LineMetrics>,
    settings: FontSettings,
}

/// Converts a ttf-parser FaceParsingError into a string.
fn convert_error(error: FaceParsingError) -> &'static str {
    use FaceParsingError::*;
    match error {
        MalformedFont => "An attempt to read out of bounds detected.",
        UnknownMagic => "Face data must start with 0x00010000, 0x74727565, 0x4F54544F or 0x74746366.",
        FaceIndexOutOfBounds => "The face index is larger than the number of faces in the font.",
        NoHeadTable => "The head table is missing or malformed.",
        NoHheaTable => "The hhea  table is missing or malformed.",
        NoMaxpTable => "The maxp table is missing or malformed.",
    }
}

impl Font {
    /// Constructs a font from an array of bytes.
    pub fn from_bytes<Data: Deref<Target = [u8]>>(data: Data, settings: FontSettings) -> FontResult<Font> {
        use ttf_parser::{Face, GlyphId, TableName};
        let face = match Face::from_slice(&data, settings.collection_index) {
            Ok(f) => f,
            Err(e) => return Err(convert_error(e)),
        };
        // TrueType and OpenType define their point order opposite of eachother.
        let reverse_points =
            if face.has_table(TableName::GlyphVariations) || face.has_table(TableName::GlyphData) {
                false
            } else {
                true
            };

        // Collect all the unique codepoint to glyph mappings.
        let mut char_to_glyph = HashMap::new();
        for subtable in face.character_mapping_subtables() {
            subtable.codepoints(|codepoint| {
                let mapping = match subtable.glyph_index(codepoint) {
                    Some(id) => id.0,
                    None => 0,
                };
                // Zero is a valid value for missing glyphs, so even if a mapping is zero, the
                // result is desireable.
                char_to_glyph.insert(codepoint, unsafe { NonZeroU16::new_unchecked(mapping) });
            });
        }

        // This is fairly degenerate, but fonts without a units per em will be assumed to have the
        // common default for compatibility.
        let units_per_em = face.units_per_em().unwrap_or(1000) as f32;

        // Parse and store all unique codepoints.
        let glyph_count = face.number_of_glyphs() as usize;
        let mut glyphs: Vec<Glyph> = vec::from_elem(Glyph::default(), glyph_count);
        for (_, mapping) in &char_to_glyph {
            let mapping = unsafe { mem::transmute::<NonZeroU16, u16>(*mapping) as usize };
            if mapping as usize >= glyph_count {
                return Err("Attempted to map a codepoint out of bounds.");
            }

            let glyph_id = GlyphId(mapping as u16);
            let glyph = &mut glyphs[mapping];
            if let Some(advance_width) = face.glyph_hor_advance(glyph_id) {
                glyph.advance_width = advance_width as f32;
            }
            if let Some(advance_height) = face.glyph_ver_advance(glyph_id) {
                glyph.advance_height = advance_height as f32;
            }

            let mut geometry = Geometry::new(settings, reverse_points);
            face.outline_glyph(glyph_id, &mut geometry);
            geometry.finalize();
            let bounds = geometry.effective_bounds;
            glyph.width = bounds.xmax - bounds.xmin;
            glyph.height = bounds.ymax - bounds.ymin;
            glyph.bounds = bounds;
            glyph.v_lines = geometry.v_lines;
            glyph.m_lines = geometry.m_lines;
        }

        // New line metrics.
        let horizontal_line_metrics =
            Some(LineMetrics::new(face.ascender(), face.descender(), face.line_gap()));
        let vertical_line_metrics = if let Some(ascender) = face.vertical_ascender() {
            Some(LineMetrics::new(
                ascender,
                face.vertical_descender().unwrap_or(0),
                face.vertical_line_gap().unwrap_or(0),
            ))
        } else {
            None
        };

        Ok(Font {
            glyphs,
            char_to_glyph,
            units_per_em,
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
            let mut offset_y = fract(1.0 - fract(height) - fract(bounds.ymin));
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
        canvas.draw(&glyph, scale, offset_x, offset_y);
        (metrics, canvas.get_bitmap())
    }

    /// Finds the internal glyph index for the given character. If the character is not present in
    /// the font then 0 is returned.
    #[inline]
    pub fn lookup_glyph_index(&self, character: char) -> usize {
        unsafe {
            mem::transmute::<Option<NonZeroU16>, u16>(self.char_to_glyph.get(&(character as u32)).copied())
                as usize
        }
    }
}
