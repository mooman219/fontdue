use crate::platform::{ceil, floor};
use crate::unicode::{linebreak_property, read_utf8, wrap_mask};
use crate::{Font, ZERO_METRICS};
use alloc::vec::*;
use core::borrow::Borrow;
use core::hash::{Hash, Hasher};

#[derive(Copy, Clone, PartialEq)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

#[derive(Copy, Clone, PartialEq)]
pub enum VerticalAlign {
    Top,
    Middle,
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

#[derive(Copy, Clone, PartialEq)]
pub struct LayoutSettings {
    /// The leftmost boundary of the text region.
    pub x: f32,
    /// The topmost boundary of the text region.
    pub y: f32,
    /// An optional rightmost boundary on the text region. A line of text that exceeds the
    /// max_width is wrapped to the line below.
    pub max_width: Option<f32>,
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

/// Hashable key that can be used to uniquely identify a rasterized glyph.
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

    fn horizontal_padding(settings: &LayoutSettings, padding: f32) -> f32 {
        if settings.max_width.is_none() {
            0.0
        } else {
            match settings.horizontal_align {
                HorizontalAlign::Left => 0.0,
                HorizontalAlign::Center => floor(padding / 2.0),
                HorizontalAlign::Right => floor(padding),
            }
        }
    }

    /// Works with &[Font] or &[&Font].
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
                    next_line.padding = max_width - (last_linebreak_x - current_x);
                    next_line.end_index = last_linebreak_index;
                    self.line_metrics.push(next_line);
                    next_line.ascent = current_ascent;
                    next_line.new_line_size = current_new_line_size;
                    last_linebreak_state = 0;
                    current_x = last_linebreak_x;
                }
                if metrics.width != 0 {
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
        next_line.padding = max_width - (caret_x - current_x);
        next_line.end_index = core::usize::MAX;
        self.line_metrics.push(next_line);

        let mut line_metrics = self.line_metrics.iter();
        let mut next_line_index = core::usize::MAX;
        let mut current_index = 0;
        let mut current_ascent = 0.0;
        let mut current_new_line_size = 0.0;
        let mut x_base = settings.x;
        let mut y_base = settings.y;
        if let Some(line) = line_metrics.next() {
            next_line_index = line.end_index;
            current_ascent = line.ascent;
            current_new_line_size = line.new_line_size;
            x_base += Self::horizontal_padding(settings, line.padding);
        }
        for glyph in output.iter_mut() {
            if current_index == next_line_index {
                if let Some(line) = line_metrics.next() {
                    x_base = settings.x - glyph.x;
                    y_base -= current_new_line_size;
                    next_line_index = line.end_index;
                    current_ascent = line.ascent;
                    current_new_line_size = line.new_line_size;
                    x_base += Self::horizontal_padding(settings, line.padding);
                }
            }
            glyph.x += x_base;
            glyph.y += y_base - current_ascent;
            current_index += 1;
        }

        // for line in &self.line_metrics {
        //     println!("{:?} {}", line, line.ascent.floor());
        // }
    }
}
