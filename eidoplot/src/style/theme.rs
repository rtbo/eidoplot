use crate::style::{color, ColorU8};
use crate::style::palette::{self, Palette};

#[derive(Debug, Clone, Copy)]
pub enum Color {
    Background,
    Foreground,
    Grid,
    LegendFill,
    LegendBorder,
    Series(palette::Color),
}

pub trait Theme {
    type Palette: Palette;

    fn is_dark(&self) -> bool;

    fn get(&self, col: Color) -> ColorU8 {
        match col {
            Color::Background => self.background(),
            Color::Foreground => self.foreground(),
            Color::Grid => self.grid(),
            Color::LegendFill => self.legend_fill(),
            Color::LegendBorder => self.legend_border(),
            Color::Series(i) => self.series(i),
        }
    }

    fn background(&self) -> ColorU8;
    fn foreground(&self) -> ColorU8;
    fn grid(&self) -> ColorU8;
    fn legend_fill(&self) -> ColorU8;
    fn legend_border(&self) -> ColorU8;

    fn palette(&self) -> &Self::Palette;

    fn series(&self, color: palette::Color) -> ColorU8 {
        self.palette().get(color)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Light<P: Palette> {
    pub palette: P,
}

impl<P: Palette> Light<P> {
    pub fn new(palette: P) -> Self {
        Light {
            palette,
        }
    }
}

impl<P: Palette> Theme for Light<P> {
    type Palette = P;

    fn is_dark(&self) -> bool {
        false
    }

    fn background(&self) -> ColorU8 {
        color::WHITE
    }

    fn foreground(&self) -> ColorU8 {
        color::BLACK
    }

    fn grid(&self) -> ColorU8 {
        ColorU8::from_rgb(128, 128, 128).with_opacity(0.6)
    }

    fn legend_fill(&self) -> ColorU8 {
        ColorU8::from_rgba(255, 255, 255, 128)
    }

    fn legend_border(&self) -> ColorU8 {
        ColorU8::from_rgb(0, 0, 0)
    }

    fn palette(&self) -> &Self::Palette {
        &self.palette
    }
}
