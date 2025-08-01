use std::io;

use eidoplot::backend::Surface;
use eidoplot::{render, style};
use svg::Node;

pub struct SvgSurface {
    doc: svg::Document,
}

impl SvgSurface {
    pub fn new(width: u32, height: u32) -> Self {
        let doc = svg::Document::new()
            .set("width", width)
            .set("height", height);
        SvgSurface { doc }
    }

    pub fn save(&self, path: &str) -> io::Result<()> {
        svg::save(path, &self.doc)
    }
}

impl Surface for SvgSurface {
    type Error = ();

    /// Prepare the surface for drawing, with the given width and height in plot units
    fn prepare(&mut self, width: f32, height: f32) -> Result<(), Self::Error> {
        self.doc.assign("viewBox", (0, 0, width, height));
        Ok(())
    }

    /// Fill the entire surface with the given color
    fn fill(&mut self, color: style::RgbaColor) -> Result<(), Self::Error> {
        let node = svg::node::element::Rectangle::new()
            .set("width", "100%")
            .set("height", "100%")
            .set("fill", color.html());
        self.doc.append(node);
        Ok(())
    }

    /// Draw a rectangle
    fn draw_rect(&mut self, rect: &render::Rect) -> Result<(), Self::Error> {
        let mut node = svg::node::element::Rectangle::new()
            .set("x", rect.rect.x)
            .set("y", rect.rect.y)
            .set("width", rect.rect.w)
            .set("height", rect.rect.h);
        if let Some(fill) = &rect.fill {
            node.assign("fill", fill.color.html());
        }
        if let Some(outline) = &rect.outline {
            let w = outline.width;
            node.assign("stroke", outline.color.html());
            node.assign("stroke-width", w);
            match outline.pattern {
                style::LinePattern::Solid => (),
                style::LinePattern::Dot => node.assign("stroke-dasharray", (w, w)),
                style::LinePattern::Dash(len, gap) => {
                    node.assign("stroke-dasharray", (w * len, w * gap))
                }
            }
        }
        self.doc.append(node);
        Ok(())
    }
}
