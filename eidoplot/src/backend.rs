use crate::{render, style};

pub trait Surface {
    type Error;

    /// Prepare the surface for drawing, with the given width and height in plot units
    fn prepare(&mut self, width: f32, height: f32) -> Result<(), Self::Error>;

    /// Fill the entire surface with the given color
    fn fill(&mut self, color: style::RgbaColor) -> Result<(), Self::Error>;

    /// Draw a rectangle
    fn draw_rect(&mut self, rect: &render::Rect) -> Result<(), Self::Error>;
}
