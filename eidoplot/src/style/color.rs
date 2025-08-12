mod named;

pub use named::*;

#[derive(Debug, Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a: 255 }
    }

    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    const fn from_html(hex: &[u8]) -> Self {
        if hex[0] != b'#' {
            panic!("Invalid hex color");
        }
        match hex.len() {
            4 => {
                let r = hex_to_u8(hex[1]);
                let g = hex_to_u8(hex[2]);
                let b = hex_to_u8(hex[3]);
                let r = r << 4 | r;
                let g = g << 4 | g;
                let b = b << 4 | b;
                Color::from_rgb(r, g, b)
            }
            5 => {
                let r = hex_to_u8(hex[1]);
                let g = hex_to_u8(hex[2]);
                let b = hex_to_u8(hex[3]);
                let a = hex_to_u8(hex[4]);
                let r = r << 4 | r;
                let g = g << 4 | g;
                let b = b << 4 | b;
                let a = a << 4 | a;
                Color::from_rgba(r, g, b, a)
            }
            7 => {
                let r = hex_to_u8(hex[1]) << 4 | hex_to_u8(hex[2]);
                let g = hex_to_u8(hex[3]) << 4 | hex_to_u8(hex[4]);
                let b = hex_to_u8(hex[5]) << 4 | hex_to_u8(hex[6]);
                Color::from_rgb(r, g, b)
            }
            9 => {
                let r = hex_to_u8(hex[1]) << 4 | hex_to_u8(hex[2]);
                let g = hex_to_u8(hex[3]) << 4 | hex_to_u8(hex[4]);
                let b = hex_to_u8(hex[5]) << 4 | hex_to_u8(hex[6]);
                let a = hex_to_u8(hex[7]) << 4 | hex_to_u8(hex[8]);
                Color::from_rgba(r, g, b, a)
            }
            _ => panic!("Invalid hex color"),
        }
    }

    pub const fn rgb(&self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }

    pub const fn rgba(&self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub const fn red(&self) -> u8 {
        self.r
    }

    pub const fn green(&self) -> u8 {
        self.g
    }

    pub const fn blue(&self) -> u8 {
        self.b
    }

    pub const fn alpha(&self) -> u8 {
        self.a
    }

    pub const fn opacity(&self) -> Option<f32> {
        if self.a == 255 {
            None
        } else {
            Some(self.a as f32 / 255.0)
        }
    }

    pub fn html(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub const fn with_red(self, r: u8) -> Self {
        Color { r, ..self }
    }

    pub const fn with_green(self, g: u8) -> Self {
        Color { g, ..self }
    }

    pub const fn with_blue(self, b: u8) -> Self {
        Color { b, ..self }
    }

    pub const fn with_alpha(self, a: u8) -> Self {
        Color { a, ..self }
    }

    pub const fn with_opacity(self, opacity: f32) -> Self {
        assert!(0.0 <= opacity && opacity <= 1.0);
        Color {
            a: (self.a as f32 * opacity) as u8,
            ..self
        }
    }

    pub const fn without_opacity(self) -> Self {
        Color { a: 255, ..self }
    }
}

const fn hex_to_u8(hex: u8) -> u8 {
    match hex {
        b'0'..=b'9' => hex - b'0',
        b'a'..=b'f' => hex - b'a' + 10,
        b'A'..=b'F' => hex - b'A' + 10,
        _ => panic!("Invalid hex character"),
    }
}
