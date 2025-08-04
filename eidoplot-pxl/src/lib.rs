use std::io;

use eidoplot::backend::Surface;
use eidoplot::{geom, render, style};

use eidoplot_svg::SvgSurface;

pub struct PxlSurface {
    width: u32,
    height: u32,
    svg: SvgSurface,
}

impl PxlSurface {
    pub fn new(width: u32, height: u32) -> Self {
        PxlSurface {
            width,
            height,
            svg: SvgSurface::new(width, height),
        }
    }

    pub fn save(&self, path: &str) -> io::Result<()> {
        use io::BufWriter;

        let mut buf = BufWriter::new(Vec::new());
        self.svg.write(&mut buf)?;
        let data = buf.into_inner()?;
        let tree = usvg::Tree::from_data(&data, &Default::default()).expect("Should be valid SVG");

        let mut pixmap = tiny_skia::Pixmap::new(self.width, self.height).unwrap();
        resvg::render(
            &tree,
            tiny_skia::Transform::identity(),
            &mut pixmap.as_mut(),
        );
        pixmap.save_png(path)?;

        Ok(())
    }
}

impl Surface for PxlSurface {
    type Error = <SvgSurface as Surface>::Error;

    fn prepare(&mut self, width: f32, height: f32) -> Result<(), Self::Error> {
        self.svg.prepare(width, height)
    }

    fn fill(&mut self, color: style::Color) -> Result<(), Self::Error> {
        self.svg.fill(color)
    }

    fn draw_rect(&mut self, rect: &render::Rect) -> Result<(), Self::Error> {
        self.svg.draw_rect(rect)
    }

    fn draw_path(&mut self, path: &render::Path) -> Result<(), Self::Error> {
        self.svg.draw_path(path)
    }

    fn push_clip_path(
        &mut self,
        path: &geom::Path,
        transform: Option<&geom::Transform>,
    ) -> Result<(), Self::Error> {
        self.svg.push_clip_path(path, transform)
    }

    fn push_clip_rect(
        &mut self,
        rect: &geom::Rect,
        transform: Option<&geom::Transform>,
    ) -> Result<(), Self::Error> {
        self.svg.push_clip_rect(rect, transform)
    }

    fn pop_clip(&mut self) -> Result<(), Self::Error> {
        self.svg.pop_clip()
    }
}
