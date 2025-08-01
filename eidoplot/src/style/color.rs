pub mod css;

pub use css::{
    AQUA, BLACK, BLUE, FUCHSIA, GRAY, GREEN, LIME, MAROON, NAVY, OLIVE, PURPLE, RED, SILVER, TEAL,
    WHITE, YELLOW,
};

#[derive(Debug, Clone, Copy)]
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RgbaColor {
    pub const fn new_rgb(r: u8, g: u8, b: u8) -> Self {
        RgbaColor { r, g, b, a: 255 }
    }

    pub const fn new_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        RgbaColor { r, g, b, a }
    }

    pub(super) const fn from_hex6(hex: &[u8; 7]) -> Self {
        if hex[0] != b'#' {
            panic!("Invalid hex color");
        }
        let r = hex_to_u8(hex[1]) << 4 | hex_to_u8(hex[2]);
        let g = hex_to_u8(hex[3]) << 4 | hex_to_u8(hex[4]);
        let b = hex_to_u8(hex[5]) << 4 | hex_to_u8(hex[6]);
        RgbaColor::new_rgb(r, g, b)
    }
}

const fn hex_to_u8(hex: u8) -> u8 {
    match hex {
        b'0'..=b'9' => hex - b'0',
        b'a'..=b'f' => hex - b'a' + 10,
        b'A'..=b'F' => hex - b'A' + 10,
        _ => panic!("Invalid hex color"),
    }
}
