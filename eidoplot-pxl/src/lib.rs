use std::{io, sync::Arc};

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

    pub fn save(&self, path: &str, fontdb: Option<Arc<fontdb::Database>>) -> io::Result<()> {
        use io::BufWriter;

        let mut buf = BufWriter::new(Vec::new());
        self.svg.write(&mut buf)?;
        let data = buf.into_inner()?;

        let mut opt = usvg::Options::default();
        if let Some(fontdb) = fontdb {
            opt.fontdb = fontdb;
        } else {
            opt.fontdb_mut().load_system_fonts();
        }

        let tree = usvg::Tree::from_data(&data, &opt).expect("Should be valid SVG");

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

impl render::Surface for PxlSurface {
    type Error = <SvgSurface as render::Surface>::Error;

    fn prepare(&mut self, size: geom::Size) -> Result<(), Self::Error> {
        self.svg.prepare(size)
    }

    fn fill(&mut self, fill: style::Fill) -> Result<(), Self::Error> {
        self.svg.fill(fill)
    }

    fn draw_rect(&mut self, rect: &render::Rect) -> Result<(), Self::Error> {
        self.svg.draw_rect(rect)
    }

    fn draw_path(&mut self, path: &render::Path) -> Result<(), Self::Error> {
        self.svg.draw_path(path)
    }

    fn draw_text(&mut self, text: &render::Text) -> Result<(), Self::Error> {
        self.svg.draw_text(text)
    }

    fn push_clip(&mut self, clip: &render::Clip) -> Result<(), Self::Error> {
        self.svg.push_clip(clip)
    }

    fn pop_clip(&mut self) -> Result<(), Self::Error> {
        self.svg.pop_clip()
    }
}
