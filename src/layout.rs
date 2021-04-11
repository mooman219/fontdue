pub use crate::unicode::CharacterData;

use crate::unicode::{classify, linebreak_property, read_utf8, wrap_mask, LINEBREAK_HARD, LINEBREAK_NONE};
use crate::Font;
use crate::{
    platform::{ceil, floor},
    Metrics,
};
use alloc::vec::*;
use core::borrow::Borrow;
use core::hash::{Hash, Hasher};

/// Horizontal alignment options for text when a max_width is provided.
#[derive(Copy, Clone, PartialEq)]
pub enum HorizontalAlign {
    /// Aligns text to the left of the region defined by the max_width.
    Left,
    /// Aligns text to the center of the region defined by the max_width.
    Center,
    /// Aligns text to the right of the region defined by the max_width.
    Right,
}

/// Vertical alignment options for text when a max_height is provided.
#[derive(Copy, Clone, PartialEq)]
pub enum VerticalAlign {
    /// Aligns text to the top of the region defined by the max_height.
    Top,
    /// Aligns text to the middle of the region defined by the max_height.
    Middle,
    /// Aligns text to the bottom of the region defined by the max_height.
    Bottom,
}

/// Wrap style is a hint for how strings of text should be wrapped to the next line. Line wrapping
/// can happen when the max width/height is reached.
#[derive(Copy, Clone, PartialEq)]
pub enum WrapStyle {
    /// Word will break lines by the Unicode line breaking algorithm (Standard Annex #14) This will
    /// generally break lines where you expect them to be broken at and will preserve words.
    Word,
    /// Letter will not preserve words, breaking into a new line after the nearest letter.
    Letter,
}

/// The direction that the Y coordinate increases in. Layout needs to be aware of your coordinate
/// system to place the glyphs correctly.
#[derive(Copy, Clone, PartialEq)]
pub enum CoordinateSystem {
    /// The Y coordinate increases up relative to the window or image. The higher up on the window,
    /// the more positive Y becomes.
    PositiveYUp,
    /// The Y coordinate increases down relative to the window or image. The lower down on the
    /// window, the more positive Y becomes.
    PositiveYDown,
}

/// Settings to configure how text layout is constrained. Text layout is considered best effort and
/// layout may violate the constraints defined here if they prevent text from being laid out.
#[derive(Copy, Clone, PartialEq)]
pub struct LayoutSettings {
    /// The leftmost boundary of the text region.
    pub x: f32,
    /// The topmost boundary of the text region.
    pub y: f32,
    /// An optional rightmost boundary on the text region. A line of text that exceeds the
    /// max_width is wrapped to the line below. If the width of a glyph is larger than the
    /// max_width, the glyph will overflow past the max_width. The application is responsible for
    /// handling the overflow.
    pub max_width: Option<f32>,
    /// An optional bottom boundary on the text region. This is used for positioning the
    /// vertical_align option. Text that exceeds the defined max_height will overflow past it. The
    /// application is responsible for handling the overflow.
    pub max_height: Option<f32>,
    /// The default is Left. This option does nothing if the max_width isn't set.
    pub horizontal_align: HorizontalAlign,
    /// The default is Top. This option does nothing if the max_height isn't set.
    pub vertical_align: VerticalAlign,
    /// The default is Word. Wrap style is a hint for how strings of text should be wrapped to the
    /// next line. Line wrapping can happen when the max width/height is reached.
    pub wrap_style: WrapStyle,
    /// The default is true. This option enables hard breaks, like new line characters, to
    /// prematurely wrap lines. If false, hard breaks will not prematurely create a new line.
    pub wrap_hard_breaks: bool,
}

impl Default for LayoutSettings {
    fn default() -> LayoutSettings {
        LayoutSettings {
            x: 0.0,
            y: 0.0,
            max_width: None,
            max_height: None,
            horizontal_align: HorizontalAlign::Left,
            vertical_align: VerticalAlign::Top,
            wrap_style: WrapStyle::Word,
            wrap_hard_breaks: true,
        }
    }
}

/// Configuration for rasterizing a glyph. This struct is also a hashable key that can be used to
/// uniquely identify a rasterized glyph for applications that want to cache glyphs.
#[derive(Debug, Copy, Clone)]
pub struct GlyphRasterConfig {
    /// The character represented by the glyph being positioned.
    pub c: char,
    /// The scale of the glyph being positioned in px.
    pub px: f32,
    /// The index of the font used in layout to raster the glyph.
    pub font_index: usize,
}

impl Hash for GlyphRasterConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.c.hash(state);
        self.px.to_bits().hash(state);
        self.font_index.hash(state);
    }
}

impl PartialEq for GlyphRasterConfig {
    fn eq(&self, other: &Self) -> bool {
        self.c == other.c && self.px == other.px && self.font_index == other.font_index
    }
}

impl Eq for GlyphRasterConfig {}

/// A positioned scaled glyph.
#[derive(Debug, Copy, Clone)]
pub struct GlyphPosition<U: Copy + Clone = ()> {
    /// Hashable key that can be used to uniquely identify a rasterized glyph.
    pub key: GlyphRasterConfig,
    /// The xmin of the glyph bounding box. This represents the left side of the glyph. Dimensions
    /// are in pixels, and are always whole numbers.
    pub x: f32,
    /// The ymin of the glyph bounding box. If your coordinate system is PositiveYUp, this
    /// represents the bottom side of the glyph. If your coordinate system is PositiveYDown, this
    /// represents the top side of the glyph. This is like this so that (y + height) always produces
    /// the other bound for the glyph.
    pub y: f32,
    /// The width of the glyph. Dimensions are in pixels.
    pub width: usize,
    /// The height of the glyph. Dimensions are in pixels.
    pub height: usize,
    /// Additional metadata associated with the character used to generate this glyph.
    pub char_data: CharacterData,
    /// Custom user data associated with the text styled used to generate this glyph.
    pub user_data: U,
    /// The byte offset into the original string used in the append call which created
    /// this glyph.
    ///
    /// At time of writing, fontdue does not handle unicode graphemes consisting of multiple
    /// code points (some letters, certain emoji's etc). This field is guaranteed to use the
    /// same logic as fontdue uses internally, even if that logic is changed.
    pub byte_offset: usize,
}

/// A style description for a segment of text.
pub struct TextStyle<'a, U: Copy + Clone = ()> {
    /// The text to layout.
    pub text: &'a str,
    /// The scale of the text in pixel units.
    pub px: f32,
    /// The font to layout the text in.
    pub font_index: usize,
    /// Additional user data to associate with glyphs produced by this text style.
    pub user_data: U,
}

impl<'a> TextStyle<'a> {
    pub fn new(text: &'a str, px: f32, font_index: usize) -> TextStyle<'a> {
        TextStyle {
            text,
            px,
            font_index,
            user_data: (),
        }
    }
}

impl<'a, U: Copy + Clone> TextStyle<'a, U> {
    pub fn with_user_data(text: &'a str, px: f32, font_index: usize, user_data: U) -> TextStyle<'a, U> {
        TextStyle {
            text,
            px,
            font_index,
            user_data,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct LineMetrics {
    /// How much empty space is left at the end of the line.
    pub padding: f32,
    /// The largest ascent for the line.
    pub ascent: f32,
    /// The largest new line size for the line.
    pub new_line_size: f32,
    /// The x position this line starts at.
    pub x_start: f32,
    /// The index of the last glyph in the line.
    pub end_index: usize,
}

/// Gives the position of a line of text after layout.
/// This can be useful to calculate where to put the caret when implementing
/// text edit functionality.
#[derive(Debug, Copy, Clone)]
pub struct LinePosition<U: Clone + Copy> {
    /// The y coordinate of the baseline of this line, in pixels
    pub baseline_y: f32,
    /// The user data for the first glyph on the line, and the byte index of the corresponding
    /// character in the original TextStyle used to create that glyph.
    /// This is useful to know to implement 'Home' button functionality in a text editor.
    ///
    /// Note that the byte index of a character is not the same as the character index, since some
    /// characters are represented by multiple bytes.
    pub line_start: (U, usize),
    /// The user data for the last glyph on the line, and the byte index following the corresponding
    /// character in the original TextStyle used to create that glyph.
    /// This is useful to know to implement 'End' button functionality in a text editor.
    /// Note that the index provided here is one past the end of the glyph. For the last glyph
    /// in a TextStyle, this will point to the byte just after the end of the string.
    ///
    /// Note that the byte index of a character is not the same as the character index, since some
    /// characters are represented by multiple bytes.
    pub line_end: (U, usize),
}

impl Default for LineMetrics {
    fn default() -> Self {
        LineMetrics {
            padding: 0.0,
            ascent: 0.0,
            x_start: 0.0,
            new_line_size: 0.0,
            end_index: 0,
        }
    }
}

/// Text layout requires a small amount of heap usage which is contained in the Layout struct. This
/// context is reused between layout calls. Reusing the Layout struct will greatly reduce memory
/// allocations and is advisable for performance.
pub struct Layout<U: Copy + Clone = ()> {
    // Global state
    flip: bool,
    // Settings state
    x: f32,
    y: f32,
    wrap_mask: u8,
    max_width: f32,
    max_height: f32,
    vertical_align: f32,
    horizontal_align: f32,
    // Single line state
    output: Vec<GlyphPosition<U>>,
    glyphs: Vec<GlyphPosition<U>>,
    line_metrics: Vec<LineMetrics>,
    line_positions: Vec<LinePosition<U>>,
    linebreak_prev: u8,
    linebreak_state: u8,
    linebreak_pos: f32,
    linebreak_idx: usize,
    current_pos: f32,
    current_ascent: f32,
    current_new_line: f32,
    current_px: f32,
    start_pos: f32,
    height: f32,
}

impl<'a, U: Copy + Clone> Layout<U> {
    /// Creates a layout instance. This requires the direction that the Y coordinate increases in.
    /// Layout needs to be aware of your coordinate system to place the glyphs correctly.
    pub fn new(coordinate_system: CoordinateSystem) -> Layout<U> {
        let mut layout = Layout {
            // Global state
            flip: coordinate_system == CoordinateSystem::PositiveYDown,
            // Settings state
            x: 0.0,
            y: 0.0,
            wrap_mask: 0,
            max_width: 0.0,
            max_height: 0.0,
            vertical_align: 0.0,
            horizontal_align: 0.0,
            // Line state
            output: Vec::new(),
            glyphs: Vec::new(),
            line_metrics: Vec::new(),
            line_positions: Vec::new(),
            linebreak_prev: 0,
            linebreak_state: 0,
            linebreak_pos: 0.0,
            linebreak_idx: 0,
            current_pos: 0.0,
            current_ascent: 0.0,
            current_new_line: 0.0,
            current_px: 0.0,
            start_pos: 0.0,
            height: 0.0,
        };
        layout.reset(&LayoutSettings::default());
        layout
    }

    /// Resets the current layout settings and clears all appended text.
    pub fn reset(&mut self, settings: &LayoutSettings) {
        self.x = settings.x;
        self.y = settings.y;
        self.wrap_mask = wrap_mask(
            settings.wrap_style == WrapStyle::Word,
            settings.wrap_hard_breaks,
            settings.max_width.is_some(),
        );
        self.max_width = settings.max_width.unwrap_or(core::f32::MAX);
        self.max_height = settings.max_height.unwrap_or(core::f32::MAX);
        self.vertical_align = if settings.max_height.is_none() {
            0.0
        } else {
            match settings.vertical_align {
                VerticalAlign::Top => 0.0,
                VerticalAlign::Middle => 0.5,
                VerticalAlign::Bottom => 1.0,
            }
        };
        self.horizontal_align = if settings.max_width.is_none() {
            0.0
        } else {
            match settings.horizontal_align {
                HorizontalAlign::Left => 0.0,
                HorizontalAlign::Center => 0.5,
                HorizontalAlign::Right => 1.0,
            }
        };
        self.clear();
    }

    /// Keeps current layout settings but clears all appended text.
    pub fn clear(&mut self) {
        self.glyphs.clear();
        self.output.clear();
        self.line_metrics.clear();
        self.line_metrics.push(LineMetrics::default());

        self.linebreak_prev = 0;
        self.linebreak_state = 0;
        self.linebreak_pos = 0.0;
        self.linebreak_idx = 0;
        self.current_pos = 0.0;
        self.current_ascent = 0.0;
        self.current_new_line = 0.0;
        self.current_px = 0.0;
        self.start_pos = 0.0;
        self.height = 0.0;
    }

    /// Gets the current height of the appended text.
    pub fn height(&self) -> f32 {
        if let Some(line) = self.line_metrics.last() {
            self.height + line.new_line_size
        } else {
            0.0
        }
    }

    /// Gets the current line count. If there's no text this still returns 1.
    pub fn lines(&self) -> usize {
        self.line_metrics.len()
    }

    /// Performs layout for text horizontally, and wrapping vertically. This makes a best effort
    /// attempt at laying out the text defined in the given styles with the provided layout
    /// settings. Text may overflow out of the bounds defined in the layout settings and it's up
    /// to the application to decide how to deal with this.
    ///
    /// Characters from the input string can only be omitted from the output, they are never
    /// reordered. The output buffer will always contain characters in the order they were defined
    /// in the styles.
    pub fn append<T: Borrow<Font>>(&mut self, fonts: &[T], style: &TextStyle<U>) {
        let mut byte_offset = 0;
        let font = &fonts[style.font_index];
        if let Some(metrics) = font.borrow().horizontal_line_metrics(style.px) {
            self.current_ascent = ceil(metrics.ascent);
            self.current_new_line = ceil(metrics.new_line_size);
            if let Some(line) = self.line_metrics.last_mut() {
                if self.current_ascent > line.ascent {
                    line.ascent = self.current_ascent;
                }
                if self.current_new_line > line.new_line_size {
                    line.new_line_size = self.current_new_line;
                }
            }
        }
        while byte_offset < style.text.len() {
            let cur_character_offset = byte_offset;
            let c = read_utf8(style.text.as_bytes(), &mut byte_offset);
            let char_index = font.borrow().lookup_glyph_index(c);
            let char_data = classify(c, char_index);
            let metrics = if !char_data.is_control() {
                font.borrow().metrics_indexed(char_index, style.px)
            } else {
                Metrics::default()
            };
            let advance = ceil(metrics.advance_width);
            let linebreak = linebreak_property(&mut self.linebreak_state, c) & self.wrap_mask;
            if linebreak >= self.linebreak_prev {
                self.linebreak_prev = linebreak;
                self.linebreak_pos = self.current_pos;
                self.linebreak_idx = self.glyphs.len();
            }
            if linebreak == LINEBREAK_HARD || (self.current_pos - self.start_pos + advance > self.max_width) {
                self.linebreak_prev = LINEBREAK_NONE;
                if let Some(line) = self.line_metrics.last_mut() {
                    line.end_index = self.linebreak_idx;
                    line.padding = self.max_width - (self.linebreak_pos - self.start_pos);
                    self.height += line.new_line_size;
                }
                self.line_metrics.push(LineMetrics {
                    padding: 0.0,
                    ascent: self.current_ascent,
                    x_start: self.linebreak_pos,
                    new_line_size: self.current_new_line,
                    end_index: 0,
                });
                self.start_pos = self.linebreak_pos;
            }
            let y = if self.flip {
                floor(-metrics.bounds.height - metrics.bounds.ymin) // PositiveYDown
            } else {
                floor(metrics.bounds.ymin) // PositiveYUp
            };
            self.glyphs.push(GlyphPosition {
                key: GlyphRasterConfig {
                    c,
                    px: style.px,
                    font_index: style.font_index,
                },
                x: self.current_pos + floor(metrics.bounds.xmin),
                y,
                width: metrics.width,
                height: metrics.height,
                char_data,
                user_data: style.user_data,
                byte_offset: cur_character_offset,
            });
            self.current_pos += advance;
        }
        if let Some(line) = self.line_metrics.last_mut() {
            line.padding = self.max_width - (self.current_pos - self.start_pos);
            line.end_index = self.glyphs.len();
        }
    }

    /// Get position data for each line in the text.
    pub fn get_lines(&mut self) -> &Vec<LinePosition<U>> {
        self.glyphs(); //Needed to make sure line_positions are up to date
        &self.line_positions
    }

    /// Gets the current laid out glyphs. Additional layout may be performed lazily here.
    pub fn glyphs(&'a mut self) -> &'a Vec<GlyphPosition<U>> {
        if self.glyphs.len() == self.output.len() {
            return &self.output;
        }

        self.line_positions.clear();
        unsafe { self.output.set_len(0) };
        self.output.reserve(self.glyphs.len());

        let dir = if self.flip {
            -1.0 // PositiveYDown
        } else {
            1.0 // PositiveYUp
        };
        let mut y = self.y - dir * floor((self.max_height - self.height()) * self.vertical_align);
        let mut idx = 0;
        for line in &self.line_metrics {
            let x = self.x - line.x_start + floor(line.padding * self.horizontal_align);

            let line_start_idx = idx;
            let baseline_y = y;
            y -= dir * line.ascent;
            while idx < line.end_index {
                let mut glyph = self.glyphs[idx];
                glyph.x += x;
                glyph.y += y;
                self.output.push(glyph);
                idx += 1;
            }
            y -= dir * (line.new_line_size - line.ascent);

            if line_start_idx < line.end_index {
                let line_start_offset = self.glyphs[line_start_idx].byte_offset;
                let line_start_userdata = self.glyphs[line_start_idx].user_data;
                let line_end_offset =
                    self.glyphs[idx - 1].byte_offset + self.glyphs[idx - 1].key.c.len_utf8();
                let line_end_userdata = self.glyphs[idx - 1].user_data;
                self.line_positions.push(LinePosition {
                    baseline_y,
                    line_start: (line_start_userdata, line_start_offset),
                    line_end: (line_end_userdata, line_end_offset),
                });
            }
        }

        &self.output
    }
}
