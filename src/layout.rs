use crate::platform::{ceil, floor};
use crate::unicode::{linebreak_property, read_utf8, wrap_mask};
use crate::{Font, ZERO_METRICS};
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
    /// The default is false. This option sets whether or not to include whitespace in the layout
    /// output. By default, whitespace is not included in the output as it's not renderable. You
    /// may want this enabled if you care about the positioning of whitespace for an interactable
    /// user interface.
    pub include_whitespace: bool,
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
            include_whitespace: false,
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
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct GlyphPosition {
    /// Hashable key that can be used to uniquely identify a rasterized glyph.
    pub key: GlyphRasterConfig,
    /// The left side of the glyph bounding box. Dimensions are in pixels, and are alawys whole
    /// numebrs.
    pub x: f32,
    /// The bottom side of the glyph bounding box. Dimensions are in pixels and are always whole
    /// numbers.
    pub y: f32,
    /// The width of the glyph. Dimensions are in pixels.
    pub width: usize,
    /// The height of the glyph. Dimensions are in pixels.
    pub height: usize,
}

/// A style description for a segment of text.
pub struct TextStyle<'a> {
    /// The text to layout.
    pub text: &'a str,
    /// The scale of the text in pixel units.
    pub px: f32,
    /// The font to layout the text in.
    pub font_index: usize,
}

impl<'a> TextStyle<'a> {
    pub fn new(text: &'a str, px: f32, font_index: usize) -> TextStyle<'a> {
        TextStyle {
            text,
            px,
            font_index,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct LineMetrics {
    pub padding: f32,
    pub ascent: f32,
    pub new_line_size: f32,
    pub end_index: usize,
}

/// Text layout requires a small amount of heap usage which is contained in the Layout struct. This
/// context is reused between layout calls. Reusing the Layout struct will greatly reduce memory
/// allocations and is advisable for performance.
pub struct Layout {
    line_metrics: Vec<LineMetrics>,
}

impl Layout {
    pub fn new() -> Layout {
        Layout {
            line_metrics: Vec::new(),
        }
    }

    fn wrap_mask_from_settings(settings: &LayoutSettings) -> u8 {
        let wrap_soft_breaks = settings.wrap_style == WrapStyle::Word;
        let wrap_hard_breaks = settings.wrap_hard_breaks;
        let wrap = settings.max_width.is_some() && (wrap_soft_breaks || wrap_hard_breaks);
        wrap_mask(wrap, wrap_soft_breaks, wrap_hard_breaks)
    }

    fn horizontal_padding(settings: &LayoutSettings, remaining_width: f32) -> f32 {
        if settings.max_width.is_none() {
            0.0
        } else {
            match settings.horizontal_align {
                HorizontalAlign::Left => 0.0,
                HorizontalAlign::Center => floor(remaining_width / 2.0),
                HorizontalAlign::Right => floor(remaining_width),
            }
        }
    }

    fn vertical_padding(settings: &LayoutSettings, height: f32) -> f32 {
        if let Some(max_height) = settings.max_height {
            if height >= max_height {
                0.0
            } else {
                match settings.vertical_align {
                    VerticalAlign::Top => 0.0,
                    VerticalAlign::Middle => floor((max_height - height) / 2.0),
                    VerticalAlign::Bottom => floor(max_height - height),
                }
            }
        } else {
            0.0
        }
    }

    /// Performs layout for text horizontally, and wrapping vertically. This makes a best effort
    /// attempt at laying out the text defined in the given styles with the provided layout
    /// settings. Text may overflow out of the bounds defined in the layout settings and it's up
    /// to the application to decide how to deal with this. Works with &[Font] or &[&Font].
    ///
    /// Characters from the input string can only be omitted from the output, they are never
    /// reordered. The output buffer will always contain characters in the order they were defined
    /// in the styles.
    pub fn layout_horizontal<T: Borrow<Font>>(
        &mut self,
        fonts: &[T],
        styles: &[&TextStyle],
        settings: &LayoutSettings,
        output: &mut Vec<GlyphPosition>,
    ) {
        // Reset internal buffers.
        unsafe {
            self.line_metrics.set_len(0);
            output.set_len(0);
        }

        // There is a lot of context.
        let wrap_mask = Self::wrap_mask_from_settings(settings); // Wrap mask based on settings.
        let max_width = settings.max_width.unwrap_or(core::f32::MAX); // The max width of the bounding box.
        let mut state: u8 = 0; // Current linebreak state.
        let mut last_linebreak_state = 0; // Last highest ranked linebreak state for the given line.
        let mut last_linebreak_x = 0.0; // X position of the last linebreak.
        let mut last_linebreak_index = 0; // Glyph position of the last linebreak.
        let mut current_x = 0.0; // Starting x for the current line.
        let mut caret_x = 0.0; // Total x for the whole text.
        let mut total_height = 0.0; // Total y for the whole text.
        let mut next_line = LineMetrics {
            padding: 0.0,
            ascent: 0.0,
            new_line_size: 0.0,
            end_index: core::usize::MAX,
        };
        let mut current_ascent = 0.0; // Ascent for the current style.
        let mut current_new_line_size = 0.0; // New line height for the current style.
        for style in styles {
            let mut byte_offset = 0;
            let font = &fonts[style.font_index];
            if let Some(metrics) = font.borrow().horizontal_line_metrics(style.px) {
                current_ascent = ceil(metrics.ascent);
                current_new_line_size = ceil(metrics.new_line_size);
                if current_ascent > next_line.ascent {
                    next_line.ascent = current_ascent;
                }
                if current_new_line_size > next_line.new_line_size {
                    next_line.new_line_size = current_new_line_size;
                }
            }
            while byte_offset < style.text.len() {
                let character = read_utf8(style.text, &mut byte_offset);
                let linebreak_state = linebreak_property(&mut state, character) & wrap_mask;
                let metrics = if character as u32 > 0x1F {
                    font.borrow().metrics(character, style.px)
                } else {
                    ZERO_METRICS
                };
                let advance = ceil(metrics.advance_width);
                if linebreak_state >= last_linebreak_state {
                    last_linebreak_state = linebreak_state;
                    last_linebreak_x = caret_x;
                    last_linebreak_index = output.len();
                }
                if caret_x - current_x + advance >= max_width || last_linebreak_state == 2 {
                    total_height += next_line.new_line_size;
                    next_line.padding = max_width - (last_linebreak_x - current_x);
                    next_line.end_index = last_linebreak_index;
                    self.line_metrics.push(next_line);
                    next_line.ascent = current_ascent;
                    next_line.new_line_size = current_new_line_size;
                    last_linebreak_state = 0;
                    current_x = last_linebreak_x;
                }
                if settings.include_whitespace || metrics.width != 0 {
                    output.push(GlyphPosition {
                        key: GlyphRasterConfig {
                            c: character,
                            px: style.px,
                            font_index: style.font_index,
                        },
                        x: caret_x + floor(metrics.bounds.xmin),
                        y: floor(metrics.bounds.ymin),
                        width: metrics.width,
                        height: metrics.height,
                    });
                }
                caret_x += advance;
            }
        }
        total_height += next_line.new_line_size;
        next_line.padding = max_width - (caret_x - current_x);
        next_line.end_index = core::usize::MAX;
        self.line_metrics.push(next_line);

        let mut line_metrics_index = 0;
        let mut next_line_index;
        let mut current_index = 0;
        let mut current_ascent;
        let mut current_new_line_size;
        let mut x_base = settings.x;
        let mut y_base = settings.y - Self::vertical_padding(settings, total_height);
        let line = self.line_metrics[0];
        next_line_index = line.end_index;
        current_ascent = line.ascent;
        current_new_line_size = line.new_line_size;
        x_base += Self::horizontal_padding(settings, line.padding);
        for glyph in output {
            if current_index == next_line_index {
                line_metrics_index += 1;
                let line = self.line_metrics[line_metrics_index];
                x_base = settings.x - glyph.x;
                y_base -= current_new_line_size;
                next_line_index = line.end_index;
                current_ascent = line.ascent;
                current_new_line_size = line.new_line_size;
                x_base += Self::horizontal_padding(settings, line.padding);
            }
            glyph.x += x_base;
            glyph.y += y_base - current_ascent;
            current_index += 1;
        }
    }
}
