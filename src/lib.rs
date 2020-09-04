mod utils;

use wasm_bindgen::prelude::*;
use fontdue::*;
use utils::*;

const ROBOTO_MONO_REGULAR_TTF: &[u8] = include_bytes!("../resources/RobotoMono-Regular.ttf");

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct ByteStream {
    offset: *const u8,
    size: usize,
}

#[wasm_bindgen]
impl ByteStream {
    pub fn new(bytes: &[u8]) -> ByteStream {
        ByteStream {
            offset: bytes.as_ptr(),
            size: bytes.len(),
        }
    }

    pub fn offset(&self) -> *const u8 {
        self.offset
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

#[wasm_bindgen]
pub struct CharRender {
    pub height: usize,
    pub width: usize,
    pub xmin: f32,
    pub ymin: f32,
    pub bitmap: ByteStream,
    #[allow(dead_code)]
    underlying: Vec<u8>
}

#[wasm_bindgen]
pub fn render(size: f32, input: char) -> CharRender {
    set_panic_hook();
    let settings = fontdue::FontSettings {
        scale: size,
        ..fontdue::FontSettings::default()
    };
    let font = Font::from_bytes(ROBOTO_MONO_REGULAR_TTF, settings).unwrap();
    let (metrics, bitmap) = font.rasterize(input, size);
    CharRender {
        height: metrics.height,
        width: metrics.width,
        xmin: metrics.bounds.xmin,
        ymin: metrics.bounds.ymin,
        bitmap: ByteStream::new(&bitmap),
        underlying: bitmap
    }
}
