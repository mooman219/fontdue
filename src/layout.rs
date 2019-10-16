use crate::Font;
use crate::FontResult;
use alloc::vec::*;

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
    pub x: f32,
    pub y: f32,
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

#[derive(Copy, Clone, PartialEq)]
pub struct GlyphPosition {
    /// The character represented by the glyph being positioned.
    pub c: char,
    /// The left side of the glyph bounding box.
    pub x: f32,
    /// The top side of the glyph bounding box.
    pub y: f32,
    /// The width of the glyph.
    pub width: f32,
    /// The height of the glyph.
    pub height: f32,
}

impl Default for GlyphPosition {
    fn default() -> GlyphPosition {
        GlyphPosition {
            c: ' ',
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

pub fn layout_horizontal(
    text: &str,
    font: &Font,
    settings: &LayoutSettings,
    output: &mut Vec<GlyphPosition>,
) {
    if !font.has_horizontal_metrics() {
        panic!("Font missing horizontal metrics");
    }
    let mut origin_x = settings.x;
    let mut origin_y = settings.y - font.new_line_height();
    for c in text.chars() {}
}

// Idea spitballing

// &[
//   TextStyle {
//     text: String,
//     color: Color,
//     scale: f32,
//     font: FontToken,
//   },
//   ..
// ]

// let mut sprites = Vec::new();
// engine.text(
//   &mut sprites,
//   TextBounds {
//     top_left: (f32, f32)
//     width: Option<f32>
//     height: Option<f32>
//   },
//   &[
//     TextStyle {}
// ])
