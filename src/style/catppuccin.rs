//! Catppuccin theme implementation
use crate::ColorU8;
use crate::style;

/// Catppuccin Latte theme
#[derive(Debug, Clone, Copy)]
pub struct Latte;

/// Catppuccin Frappe theme
#[derive(Debug, Clone, Copy)]
pub struct Frappe;

/// Catppuccin Macchiato theme
#[derive(Debug, Clone, Copy)]
pub struct Macchiato;

/// Catppuccin Mocha theme
#[derive(Debug, Clone, Copy)]
pub struct Mocha;

pub trait Flavors {
    const ROSEWATER: ColorU8;
    const FLAMINGO: ColorU8;
    const PINK: ColorU8;
    const MAUVE: ColorU8;
    const RED: ColorU8;
    const MAROON: ColorU8;
    const PEACH: ColorU8;
    const YELLOW: ColorU8;
    const GREEN: ColorU8;
    const TEAL: ColorU8;
    const SKY: ColorU8;
    const SAPPHIRE: ColorU8;
    const BLUE: ColorU8;
    const LAVENDER: ColorU8;
    const TEXT: ColorU8;
    const _SUBTEXT1: ColorU8;
    const _SUBTEXT0: ColorU8;
    const OVERLAY2: ColorU8;
    const _OVERLAY1: ColorU8;
    const _OVERLAY0: ColorU8;
    const SURFACE2: ColorU8;
    const _SURFACE1: ColorU8;
    const SURFACE0: ColorU8;
    const BASE: ColorU8;
    const _MANTLE: ColorU8;
    const _CRUST: ColorU8;
}

pub const fn theme_palette<F>() -> style::theme::ThemePalette
where
    F: Flavors,
{
    style::theme::ThemePalette {
        background: F::BASE,
        foreground: F::TEXT,
        grid: F::SURFACE2,
        legend_fill: F::SURFACE0,
        legend_border: F::OVERLAY2,
    }
}

pub const fn series_colors<F>() -> &'static [ColorU8]
where
    F: Flavors,
{
    &[
        F::BLUE,
        F::PEACH,
        F::GREEN,
        F::RED,
        F::MAUVE,
        F::MAROON,
        F::FLAMINGO,
        F::PINK,
        F::LAVENDER,
        F::TEAL,
        F::SKY,
        F::YELLOW,
        F::SAPPHIRE,
        F::ROSEWATER,
    ]
}


impl Flavors for Latte {
    const ROSEWATER: ColorU8 = ColorU8::from_html(b"#dc8a78");
    const FLAMINGO: ColorU8 = ColorU8::from_html(b"#dd7878");
    const PINK: ColorU8 = ColorU8::from_html(b"#ea76cb");
    const MAUVE: ColorU8 = ColorU8::from_html(b"#8839ef");
    const RED: ColorU8 = ColorU8::from_html(b"#d20f39");
    const MAROON: ColorU8 = ColorU8::from_html(b"#e64553");
    const PEACH: ColorU8 = ColorU8::from_html(b"#fe640b");
    const YELLOW: ColorU8 = ColorU8::from_html(b"#df8e1d");
    const GREEN: ColorU8 = ColorU8::from_html(b"#40a02b");
    const TEAL: ColorU8 = ColorU8::from_html(b"#179299");
    const SKY: ColorU8 = ColorU8::from_html(b"#04a5e5");
    const SAPPHIRE: ColorU8 = ColorU8::from_html(b"#209fb5");
    const BLUE: ColorU8 = ColorU8::from_html(b"#1e66f5");
    const LAVENDER: ColorU8 = ColorU8::from_html(b"#7287fd");
    const TEXT: ColorU8 = ColorU8::from_html(b"#4c4f69");
    const _SUBTEXT1: ColorU8 = ColorU8::from_html(b"#5c5f77");
    const _SUBTEXT0: ColorU8 = ColorU8::from_html(b"#6c6f85");
    const OVERLAY2: ColorU8 = ColorU8::from_html(b"#7c7f93");
    const _OVERLAY1: ColorU8 = ColorU8::from_html(b"#9ca0b0");
    const _OVERLAY0: ColorU8 = ColorU8::from_html(b"#c6c8d1");
    const SURFACE2: ColorU8 = ColorU8::from_html(b"#dfdfe0");
    const _SURFACE1: ColorU8 = ColorU8::from_html(b"#e8e8e8");
    const SURFACE0: ColorU8 = ColorU8::from_html(b"#f5f5f5");
    const BASE: ColorU8 = ColorU8::from_html(b"#eff1f5");
    const _MANTLE: ColorU8 = ColorU8::from_html(b"#e6e9ef");
    const _CRUST: ColorU8 = ColorU8::from_html(b"#dce0e8");
}

impl Flavors for Frappe {
    const ROSEWATER: ColorU8 = ColorU8::from_html(b"#f2d5cf");
    const FLAMINGO: ColorU8 = ColorU8::from_html(b"#eebebe");
    const PINK: ColorU8 = ColorU8::from_html(b"#f4b8e4");
    const MAUVE: ColorU8 = ColorU8::from_html(b"#ca9ee6");
    const RED: ColorU8 = ColorU8::from_html(b"#e78284");
    const MAROON: ColorU8 = ColorU8::from_html(b"#ea999c");
    const PEACH: ColorU8 = ColorU8::from_html(b"#ef9f76");
    const YELLOW: ColorU8 = ColorU8::from_html(b"#e5c890");
    const GREEN: ColorU8 = ColorU8::from_html(b"#a6d189");
    const TEAL: ColorU8 = ColorU8::from_html(b"#81c8be");
    const SKY: ColorU8 = ColorU8::from_html(b"#99d1db");
    const SAPPHIRE: ColorU8 = ColorU8::from_html(b"#85c1dc");
    const BLUE: ColorU8 = ColorU8::from_html(b"#8caaee");
    const LAVENDER: ColorU8 = ColorU8::from_html(b"#babbf1");
    const TEXT: ColorU8 = ColorU8::from_html(b"#c6d0f5");
    const _SUBTEXT1: ColorU8 = ColorU8::from_html(b"#b5bfe2");
    const _SUBTEXT0: ColorU8 = ColorU8::from_html(b"#a5adce");
    const OVERLAY2: ColorU8 = ColorU8::from_html(b"#949cbb");
    const _OVERLAY1: ColorU8 = ColorU8::from_html(b"#838ba7");
    const _OVERLAY0: ColorU8 = ColorU8::from_html(b"#737994");
    const SURFACE2: ColorU8 = ColorU8::from_html(b"#626880");
    const _SURFACE1: ColorU8 = ColorU8::from_html(b"#51576d");
    const SURFACE0: ColorU8 = ColorU8::from_html(b"#414559");
    const BASE: ColorU8 = ColorU8::from_html(b"#303446");
    const _MANTLE: ColorU8 = ColorU8::from_html(b"#292c36");
    const _CRUST: ColorU8 = ColorU8::from_html(b"#232634");
}


impl Flavors for Macchiato {
    const ROSEWATER: ColorU8 = ColorU8::from_html(b"#f4dbd6");
    const FLAMINGO: ColorU8 = ColorU8::from_html(b"#f0c6c6");
    const PINK: ColorU8 = ColorU8::from_html(b"#f5bde6");
    const MAUVE: ColorU8 = ColorU8::from_html(b"#c6a0f6");
    const RED: ColorU8 = ColorU8::from_html(b"#ed8796");
    const MAROON: ColorU8 = ColorU8::from_html(b"#ee99a0");
    const PEACH: ColorU8 = ColorU8::from_html(b"#f5a97f");
    const YELLOW: ColorU8 = ColorU8::from_html(b"#eed49f");
    const GREEN: ColorU8 = ColorU8::from_html(b"#a6da95");
    const TEAL: ColorU8 = ColorU8::from_html(b"#8bd5ca");
    const SKY: ColorU8 = ColorU8::from_html(b"#91d7e3");
    const SAPPHIRE: ColorU8 = ColorU8::from_html(b"#7dc4e4");
    const BLUE: ColorU8 = ColorU8::from_html(b"#8aadf4");
    const LAVENDER: ColorU8 = ColorU8::from_html(b"#b7bdf8");
    const TEXT: ColorU8 = ColorU8::from_html(b"#cad3f5");
    const _SUBTEXT1: ColorU8 = ColorU8::from_html(b"#b8c0e0");
    const _SUBTEXT0: ColorU8 = ColorU8::from_html(b"#a5adcb");
    const OVERLAY2: ColorU8 = ColorU8::from_html(b"#939ab7");
    const _OVERLAY1: ColorU8 = ColorU8::from_html(b"#8087a2");
    const _OVERLAY0: ColorU8 = ColorU8::from_html(b"#6e738d");
    const SURFACE2: ColorU8 = ColorU8::from_html(b"#5b6078");
    const _SURFACE1: ColorU8 = ColorU8::from_html(b"#494d64");
    const SURFACE0: ColorU8 = ColorU8::from_html(b"#363a4f");
    const BASE: ColorU8 = ColorU8::from_html(b"#24273a");
    const _MANTLE: ColorU8 = ColorU8::from_html(b"#1e2030");
    const _CRUST: ColorU8 = ColorU8::from_html(b"#181926");
}

impl Flavors for Mocha {
    const ROSEWATER: ColorU8 = ColorU8::from_html(b"#f5e0dc");
    const FLAMINGO: ColorU8 = ColorU8::from_html(b"#f2cdcd");
    const PINK: ColorU8 = ColorU8::from_html(b"#f5c2e7");
    const MAUVE: ColorU8 = ColorU8::from_html(b"#cba6f7");
    const RED: ColorU8 = ColorU8::from_html(b"#f38ba8");
    const MAROON: ColorU8 = ColorU8::from_html(b"#eba0ac");
    const PEACH: ColorU8 = ColorU8::from_html(b"#fab387");
    const YELLOW: ColorU8 = ColorU8::from_html(b"#f9e2af");
    const GREEN: ColorU8 = ColorU8::from_html(b"#a6e3a1");
    const TEAL: ColorU8 = ColorU8::from_html(b"#94e2d5");
    const SKY: ColorU8 = ColorU8::from_html(b"#89dceb");
    const SAPPHIRE: ColorU8 = ColorU8::from_html(b"#74c7ec");
    const BLUE: ColorU8 = ColorU8::from_html(b"#89b4fa");
    const LAVENDER: ColorU8 = ColorU8::from_html(b"#b4befe");
    const TEXT: ColorU8 = ColorU8::from_html(b"#cdd6f4");
    const _SUBTEXT1: ColorU8 = ColorU8::from_html(b"#bac2de");
    const _SUBTEXT0: ColorU8 = ColorU8::from_html(b"#a6adc8");
    const OVERLAY2: ColorU8 = ColorU8::from_html(b"#9399b2");
    const _OVERLAY1: ColorU8 = ColorU8::from_html(b"#7f849c");
    const _OVERLAY0: ColorU8 = ColorU8::from_html(b"#6c7086");
    const SURFACE2: ColorU8 = ColorU8::from_html(b"#585b70");
    const _SURFACE1: ColorU8 = ColorU8::from_html(b"#45475a");
    const SURFACE0: ColorU8 = ColorU8::from_html(b"#313244");
    const BASE: ColorU8 = ColorU8::from_html(b"#1e1e2e");
    const _MANTLE: ColorU8 = ColorU8::from_html(b"#181825");
    const _CRUST: ColorU8 = ColorU8::from_html(b"#11111b");
}
