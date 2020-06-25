use crate::simd::{ceil, floor};
use crate::Font;
use alloc::vec::*;
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

#[derive(Copy, Clone, PartialEq)]
pub struct LayoutSettings {
    /// The right boundary of the text region.
    pub x: f32,
    /// The topmost boundary of the text region.
    pub y: f32,
    /// An optional rightmost boundary on the text region. A line of text that exceeds the
    /// max_width is wrapped to the line below.
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
    /// If no horizontal alignment is provided, the default is Left. This option does nothing if
    /// the max_width isn't set.
    pub horizontal_align: Option<HorizontalAlign>,
    /// If no vertical alignment is provided, the default is Top. This option does nothing if the
    /// max_height field isn't set.
    pub vertical_align: Option<VerticalAlign>,
}

impl Default for LayoutSettings {
    fn default() -> LayoutSettings {
        LayoutSettings {
            x: 0.0,
            y: 0.0,
            max_width: None,
            max_height: None,
            horizontal_align: None,
            vertical_align: None,
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
    /// The horizontal subpixel offset of the glyph being positioned. Units are 1/256th of a pixel.
    pub offset: u8,
}

impl Hash for GlyphRasterConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.c.hash(state);
        self.px.to_bits().hash(state);
        self.offset.hash(state);
    }
}

impl PartialEq for GlyphRasterConfig {
    fn eq(&self, other: &Self) -> bool {
        self.c == other.c && self.offset == other.offset && self.px == other.px
    }
}

impl Eq for GlyphRasterConfig {}

/// A positioned scaled glyph.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct GlyphPosition {
    /// Hashable key that can be used to uniquely identify a rasterized glyph.
    pub key: GlyphRasterConfig,
    /// The left side of the glyph bounding box. Dimensions are in pixels, and subpixel positions
    /// are valid.
    pub x: f32,
    /// The bottom side of the glyph bounding box. Dimensions are in pixels, and subpixel positions
    /// are valid.
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
    pub font: &'a Font,
}

pub fn layout_horizontal(style: &TextStyle, settings: &LayoutSettings, output: &mut Vec<GlyphPosition>) {
    match style.font.horizontal_line_metrics(style.px) {
        None => panic!("Font missing horizontal metrics"),
        Some(metrics) => {
            let reset_x = ceil(settings.x);
            let new_line_y = ceil(metrics.new_line_size);
            let mut x_line = reset_x;
            let mut y_line = ceil(settings.y - metrics.ascent);
            for character in style.text.chars() {
                let c = style.font.metrics(character, style.px, 0.0);
                if let Some(max_width) = settings.max_width {
                    if x_line - settings.x > max_width {
                        x_line = reset_x;
                        y_line -= new_line_y;
                    }
                }
                if c.width > 0 && c.height > 0 {
                    let position = GlyphPosition {
                        key: GlyphRasterConfig {
                            c: character,
                            px: style.px,
                            offset: 0,
                        },
                        x: x_line + floor(c.bounds.xmin),
                        y: y_line + floor(c.bounds.ymin),
                        width: c.width,
                        height: c.height,
                    };
                    output.push(position);
                }
                x_line += c.advance_width;
            }
        }
    }
}
