use crate::{geom, render, style};

pub trait Surface {
    type Error;

    /// Prepare the surface for drawing, with the given width and height in plot units
    fn prepare(&mut self, width: f32, height: f32) -> Result<(), Self::Error>;

    /// Fill the entire surface with the given color
    fn fill(&mut self, color: style::Color) -> Result<(), Self::Error>;

    /// Draw a rectangle
    fn draw_rect(&mut self, rect: &render::Rect) -> Result<(), Self::Error>;

    /// Draw a path
    fn draw_path(&mut self, path: &render::Path) -> Result<(), Self::Error>;

    /// Push a clipping rectangle
    /// Subsequent draw operations will be clipped to this rectangle,
    /// until a matching [`pop_clip`] is called
    fn push_clip_rect(
        &mut self,
        rect: &geom::Rect,
        transform: Option<&geom::Transform>,
    ) -> Result<(), Self::Error> {
        self.push_clip_path(&rect.path(), transform)
    }

    /// Push a clipping path
    /// Subsequent draw operations will be clipped to this path,
    /// until a matching [`pop_clip`] is called
    fn push_clip_path(
        &mut self,
        path: &geom::Path,
        transform: Option<&geom::Transform>,
    ) -> Result<(), Self::Error>;

    /// Pop a clipping path that was pushed previously
    /// with either [`push_clip_rect`] or [`push_clip_path`]
    fn pop_clip(&mut self) -> Result<(), Self::Error>;
}
