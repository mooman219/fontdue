use crate::layout::GlyphRasterConfig;
use crate::math::{Geometry, Line};
use crate::platform::{as_i32, ceil, floor, fract, is_negative};
use crate::raster::Raster;
use crate::table::{load_gsub, TableKern};
use crate::unicode;
use crate::FontResult;
use crate::{HashMap, HashSet};
use alloc::string::String;
use alloc::vec;
use alloc::vec::*;
use core::hash::{Hash, Hasher};
use core::mem;
use core::num::NonZeroU16;
use core::ops::Deref;
use ttf_parser::{Face, FaceParsingError, GlyphId, Tag};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Defines the bounds for a glyph's outline in subpixels. A glyph's outline is always contained in
/// its bitmap.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct OutlineBounds {
    /// Subpixel offset of the left-most edge of the glyph's outline.
    pub xmin: f32,
    /// Subpixel offset of the bottom-most edge of the glyph's outline.
    pub ymin: f32,
    /// The width of the outline in subpixels.
    pub width: f32,
    /// The height of the outline in subpixels.
    pub height: f32,
}

impl Default for OutlineBounds {
    fn default() -> Self {
        Self {
            xmin: 0.0,
            ymin: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

impl OutlineBounds {
    /// Scales the bounding box by the given factor.
    #[inline(always)]
    pub fn scale(&self, scale: f32) -> OutlineBounds {
        OutlineBounds {
            xmin: self.xmin * scale,
            ymin: self.ymin * scale,
            width: self.width * scale,
            height: self.height * scale,
        }
    }
}

/// Encapsulates all layout information associated with a glyph for a fixed scale.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Metrics {
    /// Whole pixel offset of the left-most edge of the bitmap. This may be negative to reflect the
    /// glyph is positioned to the left of the origin.
    pub xmin: i32,
    /// Whole pixel offset of the bottom-most edge of the bitmap. This may be negative to reflect
    /// the glyph is positioned below the baseline.
    pub ymin: i32,
    /// The width of the bitmap in whole pixels.
    pub width: usize,
    /// The height of the bitmap in whole pixels.
    pub height: usize,
    /// Advance width of the glyph in subpixels. Used in horizontal fonts.
    pub advance_width: f32,
    /// Advance height of the glyph in subpixels. Used in vertical fonts.
    pub advance_height: f32,
    /// The bounding box that contains the glyph's outline at the offsets specified by the font.
    /// This is always a smaller box than the bitmap bounds.
    pub bounds: OutlineBounds,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            xmin: 0,
            ymin: 0,
            width: 0,
            height: 0,
            advance_width: 0.0,
            advance_height: 0.0,
            bounds: OutlineBounds::default(),
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

/// Stores compiled geometry and metric information.
#[derive(Clone)]
pub(crate) struct Glyph {
    pub v_lines: Vec<Line>,
    pub m_lines: Vec<Line>,
    advance_width: f32,
    advance_height: f32,
    pub bounds: OutlineBounds,
}

impl Default for Glyph {
    fn default() -> Self {
        Glyph {
            v_lines: Vec::new(),
            m_lines: Vec::new(),
            advance_width: 0.0,
            advance_height: 0.0,
            bounds: OutlineBounds::default(),
        }
    }
}

/// Settings for controlling specific font and layout behavior.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct FontSettings {
    /// The default is 0. The index of the font to use if parsing a font collection.
    pub collection_index: u32,
    /// The default is 40. The scale in px the font geometry is optimized for. Fonts rendered at
    /// the scale defined here will be the most optimal in terms of looks and performance. Glyphs
    /// rendered smaller than this scale will look the same but perform slightly worse, while
    /// glyphs rendered larger than this will looks worse but perform slightly better. The units of
    /// the scale are pixels per Em unit.
    pub scale: f32,
    /// The default is true. If enabled, will load glyphs for substitutions (liagtures, etc.) from
    /// the gsub table on compatible fonts. Only makes a difference when using indexed operations,
    /// i.e. `Font::raserize_indexed`, as singular characters do not have enough context to be
    /// substituted.
    pub load_substitutions: bool,
}

impl Default for FontSettings {
    fn default() -> FontSettings {
        FontSettings {
            collection_index: 0,
            scale: 40.0,
            load_substitutions: true,
        }
    }
}

/// Represents a font. Fonts are immutable after creation and owns its own copy of the font data.
#[derive(Clone)]
pub struct Font {
    name: Option<String>,
    units_per_em: f32,
    glyphs: Vec<Glyph>,
    char_to_glyph: HashMap<char, NonZeroU16>,
    horizontal_line_metrics: Option<LineMetrics>,
    horizontal_kern: Option<HashMap<u32, i16>>,
    vertical_line_metrics: Option<LineMetrics>,
    settings: FontSettings,
    hash: usize,
}

impl Hash for Font {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl core::fmt::Debug for Font {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Font")
            .field("name", &self.name)
            .field("settings", &self.settings)
            .field("units_per_em", &self.units_per_em)
            .field("hash", &self.hash)
            .finish()
    }
}

/// Converts a ttf-parser FaceParsingError into a string.
fn convert_error(error: FaceParsingError) -> &'static str {
    use FaceParsingError::*;
    match error {
        MalformedFont => "An attempt to read out of bounds detected.",
        UnknownMagic => "Face data must start with 0x00010000, 0x74727565, 0x4F54544F or 0x74746366.",
        FaceIndexOutOfBounds => "The face index is larger than the number of faces in the font.",
        NoHeadTable => "The head table is missing or malformed.",
        NoHheaTable => "The hhea table is missing or malformed.",
        NoMaxpTable => "The maxp table is missing or malformed.",
    }
}

fn convert_name(face: &Face) -> Option<String> {
    for name in face.names() {
        if name.name_id == 4 && name.is_unicode() {
            return Some(unicode::decode_utf16(name.name));
        }
    }
    None
}

impl Font {
    /// Constructs a font from an array of bytes.
    pub fn from_bytes<Data: Deref<Target = [u8]>>(data: Data, settings: FontSettings) -> FontResult<Font> {
        let hash = crate::hash::hash(&data);

        let face = match Face::parse(&data, settings.collection_index) {
            Ok(f) => f,
            Err(e) => return Err(convert_error(e)),
        };
        let name = convert_name(&face);

        // Optionally get kerning values for the font. This should be a try block in the future.
        let horizontal_kern: Option<HashMap<u32, i16>> = (|| {
            let table: &[u8] = face.raw_face().table(Tag::from_bytes(&b"kern"))?;
            let table: TableKern = TableKern::new(table)?;
            Some(table.horizontal_mappings)
        })();

        // Collect all the unique codepoint to glyph mappings.
        let glyph_count = face.number_of_glyphs();
        let mut indices_to_load = HashSet::with_capacity(glyph_count as usize);
        let mut char_to_glyph = HashMap::with_capacity(glyph_count as usize);
        indices_to_load.insert(0u16);
        if let Some(subtable) = face.tables().cmap {
            for subtable in subtable.subtables {
                subtable.codepoints(|codepoint| {
                    if let Some(mapping) = subtable.glyph_index(codepoint) {
                        if let Some(mapping) = NonZeroU16::new(mapping.0) {
                            indices_to_load.insert(mapping.get());
                            char_to_glyph.insert(unsafe { mem::transmute::<u32, char>(codepoint) }, mapping);
                        }
                    }
                })
            }
        }

        // If the gsub table exists and the user needs it, add all of its glyphs to the glyphs we should load.
        if settings.load_substitutions {
            load_gsub(&face, &mut indices_to_load);
        }

        let units_per_em = face.units_per_em() as f32;

        // Parse and store all unique codepoints.
        let mut glyphs: Vec<Glyph> = vec::from_elem(Glyph::default(), glyph_count as usize);

        let generate_glyph = |index: u16| -> Result<Glyph, &'static str> {
            if index >= glyph_count {
                return Err("Attempted to map a codepoint out of bounds.");
            }

            let mut glyph = Glyph::default();
            let glyph_id = GlyphId(index);
            if let Some(advance_width) = face.glyph_hor_advance(glyph_id) {
                glyph.advance_width = advance_width as f32;
            }
            if let Some(advance_height) = face.glyph_ver_advance(glyph_id) {
                glyph.advance_height = advance_height as f32;
            }

            let mut geometry = Geometry::new(settings.scale, units_per_em);
            face.outline_glyph(glyph_id, &mut geometry);
            geometry.finalize(&mut glyph);
            Ok(glyph)
        };

        #[cfg(not(feature = "parallel"))]
        for index in indices_to_load {
            glyphs[index as usize] = generate_glyph(index)?;
        }

        #[cfg(feature = "parallel")]
        {
            let generated: Vec<(u16, Glyph)> = indices_to_load
                .into_par_iter()
                .map(|index| Ok((index, generate_glyph(index)?)))
                .collect::<Result<_, _>>()?;
            for (index, glyph) in generated {
                glyphs[index as usize] = glyph;
            }
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
            name,
            glyphs,
            char_to_glyph,
            units_per_em,
            horizontal_line_metrics,
            horizontal_kern,
            vertical_line_metrics,
            settings,
            hash,
        })
    }

    /// Returns the font's face name if it has one. It is from `Name ID 4` (Full Name) in the name table.
    /// See https://learn.microsoft.com/en-us/typography/opentype/spec/name#name-ids for more info.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns all valid unicode codepoints that have mappings to glyph geometry in the font, along
    /// with their associated index. This does not include grapheme cluster mappings. The mapped
    /// NonZeroU16 index can be used in the _indexed font functions.
    pub fn chars(&self) -> &HashMap<char, NonZeroU16> {
        &self.char_to_glyph
    }

    /// Returns a precomputed hash for the font file.
    pub fn file_hash(&self) -> usize {
        self.hash
    }

    /// New line metrics for fonts that append characters to lines horizontally, and append new
    /// lines vertically (above or below the current line). Only populated for fonts with the
    /// appropriate metrics, none if it's missing.
    /// # Arguments
    ///
    /// * `px` - The size to scale the line metrics by. The units of the scale are pixels per Em
    /// unit.
    pub fn horizontal_line_metrics(&self, px: f32) -> Option<LineMetrics> {
        let metrics = self.horizontal_line_metrics?;
        Some(metrics.scale(self.scale_factor(px)))
    }

    /// New line metrics for fonts that append characters to lines vertically, and append new
    /// lines horizontally (left or right of the current line). Only populated for fonts with the
    /// appropriate metrics, none if it's missing.
    /// # Arguments
    ///
    /// * `px` - The size to scale the line metrics by. The units of the scale are pixels per Em
    /// unit.
    pub fn vertical_line_metrics(&self, px: f32) -> Option<LineMetrics> {
        let metrics = self.vertical_line_metrics?;
        Some(metrics.scale(self.scale_factor(px)))
    }

    /// Gets the font's units per em.
    #[inline(always)]
    pub fn units_per_em(&self) -> f32 {
        self.units_per_em
    }

    /// Calculates the glyph's outline scale factor for a given px size. The units of the scale are
    /// pixels per Em unit.
    #[inline(always)]
    pub fn scale_factor(&self, px: f32) -> f32 {
        px / self.units_per_em
    }

    /// Retrieves the horizontal scaled kerning value for two adjacent characters.
    /// # Arguments
    ///
    /// * `left` - The character on the left hand side of the pairing.
    /// * `right` - The character on the right hand side of the pairing.
    /// * `px` - The size to scale the kerning value for. The units of the scale are pixels per Em
    /// unit.
    /// # Returns
    ///
    /// * `Option<f32>` - The horizontal scaled kerning value if one is present in the font for the
    /// given left and right pair, None otherwise.
    #[inline(always)]
    pub fn horizontal_kern(&self, left: char, right: char, px: f32) -> Option<f32> {
        self.horizontal_kern_indexed(self.lookup_glyph_index(left), self.lookup_glyph_index(right), px)
    }

    /// Retrieves the horizontal scaled kerning value for two adjacent glyph indicies.
    /// # Arguments
    ///
    /// * `left` - The glyph index on the left hand side of the pairing.
    /// * `right` - The glyph index on the right hand side of the pairing.
    /// * `px` - The size to scale the kerning value for. The units of the scale are pixels per Em
    /// unit.
    /// # Returns
    ///
    /// * `Option<f32>` - The horizontal scaled kerning value if one is present in the font for the
    /// given left and right pair, None otherwise.
    #[inline(always)]
    pub fn horizontal_kern_indexed(&self, left: u16, right: u16, px: f32) -> Option<f32> {
        let scale = self.scale_factor(px);
        let map = self.horizontal_kern.as_ref()?;
        let key = u32::from(left) << 16 | u32::from(right);
        let value = map.get(&key)?;
        Some((*value as f32) * scale)
    }

    /// Retrieves the layout metrics for the given character. If the character isn't present in the
    /// font, then the layout for the font's default character is returned instead.
    /// # Arguments
    ///
    /// * `index` - The character in the font to to generate the layout metrics for.
    /// * `px` - The size to generate the layout metrics for the character at. Cannot be negative.
    /// The units of the scale are pixels per Em unit.
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
    /// * `px` - The size to generate the layout metrics for the glyph at. Cannot be negative. The
    /// units of the scale are pixels per Em unit.
    /// # Returns
    ///
    /// * `Metrics` - Sizing and positioning metadata for the glyph.
    pub fn metrics_indexed(&self, index: u16, px: f32) -> Metrics {
        let glyph = &self.glyphs[index as usize];
        let scale = self.scale_factor(px);
        let (metrics, _, _) = self.metrics_raw(scale, glyph, 0.0);
        metrics
    }

    /// Internal function to generate the metrics, offset_x, and offset_y of the glyph.
    fn metrics_raw(&self, scale: f32, glyph: &Glyph, offset: f32) -> (Metrics, f32, f32) {
        let bounds = glyph.bounds.scale(scale);
        let mut offset_x = fract(bounds.xmin + offset);
        let mut offset_y = fract(1.0 - fract(bounds.height) - fract(bounds.ymin));
        if is_negative(offset_x) {
            offset_x += 1.0;
        }
        if is_negative(offset_y) {
            offset_y += 1.0;
        }
        let metrics = Metrics {
            xmin: as_i32(floor(bounds.xmin)),
            ymin: as_i32(floor(bounds.ymin)),
            width: as_i32(ceil(bounds.width + offset_x)) as usize,
            height: as_i32(ceil(bounds.height + offset_y)) as usize,
            advance_width: scale * glyph.advance_width,
            advance_height: scale * glyph.advance_height,
            bounds,
        };
        (metrics, offset_x, offset_y)
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
        self.rasterize_indexed(config.glyph_index, config.px)
    }

    /// Retrieves the layout metrics and rasterized bitmap for the given character. If the
    /// character isn't present in the font, then the layout and bitmap for the font's default
    /// character is returned instead.
    /// # Arguments
    ///
    /// * `character` - The character to rasterize.
    /// * `px` - The size to render the character at. Cannot be negative. The units of the scale
    /// are pixels per Em unit.
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

    /// Retrieves the layout rasterized bitmap for the given raster config. If the raster config's
    /// character isn't present in the font, then the layout and bitmap for the font's default
    /// character's raster is returned instead.
    ///
    /// This will perform the operation with the width multiplied by 3, as to simulate subpixels.
    /// Taking these as RGB values will perform subpixel anti aliasing.
    /// # Arguments
    ///
    /// * `config` - The settings to render the character at.
    /// # Returns
    ///
    /// * `Metrics` - Sizing and positioning metadata for the rasterized glyph.
    /// * `Vec<u8>` - Swizzled RGB coverage vector for the glyph. Coverage is a linear scale where 0
    /// represents 0% coverage of that subpixel by the glyph and 255 represents 100% coverage. The
    /// vec starts at the top left corner of the glyph.
    #[inline]
    pub fn rasterize_config_subpixel(&self, config: GlyphRasterConfig) -> (Metrics, Vec<u8>) {
        self.rasterize_indexed_subpixel(config.glyph_index, config.px)
    }

    /// Retrieves the layout metrics and rasterized bitmap for the given character. If the
    /// character isn't present in the font, then the layout and bitmap for the font's default
    /// character is returned instead.
    ///
    /// This will perform the operation with the width multiplied by 3, as to simulate subpixels.
    /// Taking these as RGB values will perform subpixel anti aliasing.
    /// # Arguments
    ///
    /// * `character` - The character to rasterize.
    /// * `px` - The size to render the character at. Cannot be negative. The units of the scale
    /// are pixels per Em unit.
    /// # Returns
    ///
    /// * `Metrics` - Sizing and positioning metadata for the rasterized glyph.
    /// * `Vec<u8>` - Swizzled RGB coverage vector for the glyph. Coverage is a linear scale where 0
    /// represents 0% coverage of that subpixel by the glyph and 255 represents 100% coverage. The
    /// vec starts at the top left corner of the glyph.
    #[inline]
    pub fn rasterize_subpixel(&self, character: char, px: f32) -> (Metrics, Vec<u8>) {
        self.rasterize_indexed_subpixel(self.lookup_glyph_index(character), px)
    }

    /// Retrieves the layout metrics and rasterized bitmap at the given index. You normally want to
    /// be using rasterize(char, f32) instead, unless your glyphs are pre-indexed.
    /// # Arguments
    ///
    /// * `index` - The glyph index in the font to rasterize.
    /// * `px` - The size to render the character at. Cannot be negative. The units of the scale
    /// are pixels per Em unit.
    /// # Returns
    ///
    /// * `Metrics` - Sizing and positioning metadata for the rasterized glyph.
    /// * `Vec<u8>` - Coverage vector for the glyph. Coverage is a linear scale where 0 represents
    /// 0% coverage of that pixel by the glyph and 255 represents 100% coverage. The vec starts at
    /// the top left corner of the glyph.
    pub fn rasterize_indexed(&self, index: u16, px: f32) -> (Metrics, Vec<u8>) {
        if px <= 0.0 {
            return (Metrics::default(), Vec::new());
        }
        let glyph = &self.glyphs[index as usize];
        let scale = self.scale_factor(px);
        let (metrics, offset_x, offset_y) = self.metrics_raw(scale, glyph, 0.0);
        let mut canvas = Raster::new(metrics.width, metrics.height);
        canvas.draw(&glyph, scale, scale, offset_x, offset_y);
        (metrics, canvas.get_bitmap())
    }

    /// Retrieves the layout metrics and rasterized bitmap at the given index. You normally want to
    /// be using rasterize(char, f32) instead, unless your glyphs are pre-indexed.
    ///
    /// This will perform the operation with the width multiplied by 3, as to simulate subpixels.
    /// Taking these as RGB values will perform subpixel anti aliasing.
    /// # Arguments
    ///
    /// * `index` - The glyph index in the font to rasterize.
    /// * `px` - The size to render the character at. Cannot be negative. The units of the scale
    /// are pixels per Em unit.
    /// # Returns
    ///
    /// * `Metrics` - Sizing and positioning metadata for the rasterized glyph.
    /// * `Vec<u8>` - Swizzled RGB coverage vector for the glyph. Coverage is a linear scale where 0
    /// represents 0% coverage of that subpixel by the glyph and 255 represents 100% coverage. The
    /// vec starts at the top left corner of the glyph.
    pub fn rasterize_indexed_subpixel(&self, index: u16, px: f32) -> (Metrics, Vec<u8>) {
        if px <= 0.0 {
            return (Metrics::default(), Vec::new());
        }
        let glyph = &self.glyphs[index as usize];
        let scale = self.scale_factor(px);
        let (metrics, offset_x, offset_y) = self.metrics_raw(scale, glyph, 0.0);
        let mut canvas = Raster::new(metrics.width * 3, metrics.height);
        canvas.draw(&glyph, scale * 3.0, scale, offset_x, offset_y);
        (metrics, canvas.get_bitmap())
    }

    /// Checks if the font has a glyph for the given character.
    #[inline]
    pub fn has_glyph(&self, character: char) -> bool {
        self.lookup_glyph_index(character) != 0
    }

    /// Finds the internal glyph index for the given character. If the character is not present in
    /// the font then 0 is returned.
    #[inline]
    pub fn lookup_glyph_index(&self, character: char) -> u16 {
        // This is safe, Option<NonZeroU16> is documented to have the same layout as u16.
        unsafe { mem::transmute::<Option<NonZeroU16>, u16>(self.char_to_glyph.get(&character).copied()) }
    }

    /// Gets the total glyphs in the font.
    pub fn glyph_count(&self) -> u16 {
        self.glyphs.len() as u16
    }
}
