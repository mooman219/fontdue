pub mod baseline_tests;
pub mod letter_render_tests;

pub static FONT_NAMES: [&str; 8] = [
    "Roboto-Regular",
    "RobotoMono-Regular",
    "Comfortaa-Regular",
    "Inconsolata-Regular",
    "FasterOne-Regular",
    "Exo2-Regular",
    "GreatVibes-Regular",
    "modernpics",
];
pub static FONTS: [&[u8]; 8] = [
    include_bytes!("../../resources/fonts/Roboto-Regular.ttf"),
    include_bytes!("../../resources/fonts/RobotoMono-Regular.ttf"),
    include_bytes!("../../resources/fonts/Comfortaa-Regular.ttf"),
    include_bytes!("../../resources/fonts/Inconsolata-Regular.ttf"),
    include_bytes!("../../resources/fonts/FasterOne-Regular.ttf"),
    include_bytes!("../../resources/fonts/Exo2-Regular.otf"),
    include_bytes!("../../resources/fonts/GreatVibes-Regular.otf"),
    include_bytes!("../../resources/fonts/modernpics.otf"),
];
