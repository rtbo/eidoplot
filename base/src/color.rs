use std::str::FromStr;
use std::{error, fmt};

mod named;

pub use named::*;

pub trait ResolveColor<Color> {
    fn resolve_color(&self, color: &Color) -> ColorU8;
}

pub trait Color: Clone + Copy {
    #[inline]
    fn resolve<R>(&self, rc: &R) -> ColorU8
    where
        R: ResolveColor<Self>,
        Self: Sized,
    {
        rc.resolve_color(self)
    }
}

impl Color for ColorU8 {}

impl ResolveColor<ColorU8> for () {
    fn resolve_color(&self, color: &ColorU8) -> ColorU8 {
        *color
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorU8 {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl ColorU8 {
    pub const fn from_rgb_f32(r: f32, g: f32, b: f32) -> Self {
        let r = (r.clamp(0.0, 1.0) * 255.0) as u8;
        let g = (g.clamp(0.0, 1.0) * 255.0) as u8;
        let b = (b.clamp(0.0, 1.0) * 255.0) as u8;
        ColorU8 { r, g, b, a: 255 }
    }

    pub const fn from_rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Self {
        let r = (r.clamp(0.0, 1.0) * 255.0) as u8;
        let g = (g.clamp(0.0, 1.0) * 255.0) as u8;
        let b = (b.clamp(0.0, 1.0) * 255.0) as u8;
        let a = (a.clamp(0.0, 1.0) * 255.0) as u8;
        ColorU8 { r, g, b, a }
    }

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        ColorU8 { r, g, b, a: 255 }
    }

    pub const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        ColorU8 { r, g, b, a }
    }

    pub const fn from_html(hex: &[u8]) -> Self {
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
                ColorU8::from_rgb(r, g, b)
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
                ColorU8::from_rgba(r, g, b, a)
            }
            7 => {
                let r = hex_to_u8(hex[1]) << 4 | hex_to_u8(hex[2]);
                let g = hex_to_u8(hex[3]) << 4 | hex_to_u8(hex[4]);
                let b = hex_to_u8(hex[5]) << 4 | hex_to_u8(hex[6]);
                ColorU8::from_rgb(r, g, b)
            }
            9 => {
                let r = hex_to_u8(hex[1]) << 4 | hex_to_u8(hex[2]);
                let g = hex_to_u8(hex[3]) << 4 | hex_to_u8(hex[4]);
                let b = hex_to_u8(hex[5]) << 4 | hex_to_u8(hex[6]);
                let a = hex_to_u8(hex[7]) << 4 | hex_to_u8(hex[8]);
                ColorU8::from_rgba(r, g, b, a)
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

    pub const fn rgb_f32(&self) -> [f32; 3] {
        [self.r as f32 / 255.0, self.g as f32 / 255.0, self.b as f32 / 255.0]
    }

    pub const fn rgba_f32(&self) -> [f32; 4] {
        [self.r as f32 / 255.0, self.g as f32 / 255.0, self.b as f32 / 255.0, self.a as f32 / 255.0]
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
        ColorU8 { r, ..self }
    }

    pub const fn with_green(self, g: u8) -> Self {
        ColorU8 { g, ..self }
    }

    pub const fn with_blue(self, b: u8) -> Self {
        ColorU8 { b, ..self }
    }

    pub const fn with_alpha(self, a: u8) -> Self {
        ColorU8 { a, ..self }
    }

    pub const fn with_opacity(self, opacity: f32) -> Self {
        assert!(0.0 <= opacity && opacity <= 1.0);
        ColorU8 {
            a: (self.a as f32 * opacity) as u8,
            ..self
        }
    }

    pub const fn without_opacity(self) -> Self {
        ColorU8 { a: 255, ..self }
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

/// Erreur de parsing pour ColorU8
#[derive(Debug)]
pub enum ParseError {
    InvalidFormat,
    InvalidComponent,
    InvalidAlphaComponent,
    InvalidHex,
    UnknownName,
    IntError,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::InvalidFormat => write!(f, "invalid color format"),
            ParseError::InvalidComponent => write!(f, "invalid color component"),
            ParseError::InvalidAlphaComponent => write!(f, "invalid alpha component"),
            ParseError::InvalidHex => write!(f, "invalid hex color"),
            ParseError::UnknownName => write!(f, "unknown color name"),
            ParseError::IntError => write!(f, "integer parse error"),
        }
    }
}

impl error::Error for ParseError {}

fn parse_component_0_255(s: &str) -> Result<u8, ParseError> {
    let s = s.trim();
    if s.ends_with('%') {
        let val = s[..s.len() - 1]
            .trim()
            .parse::<f32>()
            .map_err(|_| ParseError::InvalidComponent)?;
        if !(0.0..=100.0).contains(&val) {
            return Err(ParseError::InvalidComponent);
        }
        Ok(((val / 100.0) * 255.0).round().clamp(0.0, 255.0) as u8)
    } else {
        // integer 0-255
        let v: i32 = s.parse().map_err(|_| ParseError::InvalidComponent)?;
        if !(0..=255).contains(&v) {
            return Err(ParseError::InvalidComponent);
        }
        Ok(v as u8)
    }
}

fn parse_alpha(s: &str) -> Result<u8, ParseError> {
    let s = s.trim();
    if s.ends_with('%') {
        // percentage alpha 0-100%
        let val = s[..s.len() - 1]
            .trim()
            .parse::<f32>()
            .map_err(|_| ParseError::InvalidAlphaComponent)?;
        if !(0.0..=100.0).contains(&val) {
            return Err(ParseError::InvalidAlphaComponent);
        }
        Ok(((val / 100.0) * 255.0).round().clamp(0.0, 255.0) as u8)
    } else {
        // try float 0.0-1.0
        if let Ok(f) = s.parse::<f32>() {
            if !(0.0..=1.0).contains(&f) {
                return Err(ParseError::InvalidAlphaComponent);
            }
            return Ok((f * 255.0).round().clamp(0.0, 255.0) as u8);
        }
        // try integer 0-255
        let v: i32 = s.parse().map_err(|_| ParseError::InvalidAlphaComponent)?;
        if !(0..=255).contains(&v) {
            return Err(ParseError::InvalidAlphaComponent);
        }
        Ok(v as u8)
    }
}

impl FromStr for ColorU8 {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw = s.trim();
        if raw.is_empty() {
            return Err(ParseError::InvalidFormat);
        }

        // HTML hex: starts with '#'
        if raw.starts_with('#') {
            // safe to call from_html, but we should validate length first
            let bytes = raw.as_bytes();
            match bytes.len() {
                4 | 5 | 7 | 9 => {
                    // from_html panics if first char != '#', but we checked it
                    Ok(ColorU8::from_html(bytes))
                }
                _ => Err(ParseError::InvalidHex),
            }
        }
        // rgb(...) or rgba(...)
        else if raw.to_ascii_lowercase().starts_with("rgb(") && raw.ends_with(')') {
            let inner = &raw[4..raw.len() - 1];
            let parts: Vec<&str> = inner.split(',').collect();
            if parts.len() != 3 {
                return Err(ParseError::InvalidFormat);
            }
            let r = parse_component_0_255(parts[0])?;
            let g = parse_component_0_255(parts[1])?;
            let b = parse_component_0_255(parts[2])?;
            Ok(ColorU8::from_rgb(r, g, b))
        } else if raw.to_ascii_lowercase().starts_with("rgba(") && raw.ends_with(')') {
            let inner = &raw[5..raw.len() - 1];
            let parts: Vec<&str> = inner.split(',').collect();
            if parts.len() != 4 {
                return Err(ParseError::InvalidFormat);
            }
            let r = parse_component_0_255(parts[0])?;
            let g = parse_component_0_255(parts[1])?;
            let b = parse_component_0_255(parts[2])?;
            let a = parse_alpha(parts[3])?;
            Ok(ColorU8::from_rgba(r, g, b, a))
        }
        // named color
        else {
            if let Some(col) = named::lookup_name(raw) {
                Ok(col)
            } else {
                Err(ParseError::UnknownName)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_html_hex() {
        // full and short hex
        assert_eq!("#ff0000".parse::<ColorU8>().unwrap(), RED);
        assert_eq!("#f00".parse::<ColorU8>().unwrap(), RED);

        // hex with alpha
        let c = "#ff000080".parse::<ColorU8>().unwrap();
        assert_eq!(c.rgba(), [255, 0, 0, 128]);
    }

    #[test]
    fn parse_css_rgb_rgba() {
        // integer rgb
        assert_eq!("rgb(255,0,0)".parse::<ColorU8>().unwrap(), RED);

        // percentage rgb
        assert_eq!("rgb(100%,0%,0%)".parse::<ColorU8>().unwrap(), RED);

        // rgba with float alpha
        let c = "rgba(255,0,0,0.5)".parse::<ColorU8>().unwrap();
        assert_eq!(c.rgba(), [255, 0, 0, 128]);

        // rgba with percentage alpha
        let c2 = "rgba(255,0,0,50%)".parse::<ColorU8>().unwrap();
        assert_eq!(c2.rgba(), [255, 0, 0, 128]);

        // rgba with float alpha 0.0-1.0
        let c3 = "rgba(255, 0, 0, 0.5)".parse::<ColorU8>().unwrap();
        assert_eq!(c3.rgba(), [255, 0, 0, 128]);
    }

    #[test]
    fn parse_named_colors() {
        // simple name
        assert_eq!("red".parse::<ColorU8>().unwrap(), RED);

        // case-insensitive
        assert_eq!("AliceBlue".parse::<ColorU8>().unwrap(), ALICEBLUE);
    }

    #[test]
    fn parse_errors() {
        // empty
        assert!(matches!(
            "".parse::<ColorU8>(),
            Err(ParseError::InvalidFormat)
        ));

        // invalid hex length
        assert!(matches!(
            "#12345".parse::<ColorU8>(),
            Err(ParseError::InvalidHex)
        ));

        // invalid rgb component (out of 0-255)
        assert!(matches!(
            "rgb(300,0,0)".parse::<ColorU8>(),
            Err(ParseError::InvalidComponent)
        ));

        // invalid rgba alpha (float > 1.0)
        assert!(matches!(
            "rgba(255,0,0,2.0)".parse::<ColorU8>(),
            Err(ParseError::InvalidAlphaComponent)
        ));

        // unknown name
        assert!(matches!(
            "notacolor".parse::<ColorU8>(),
            Err(ParseError::UnknownName)
        ));
    }
}
