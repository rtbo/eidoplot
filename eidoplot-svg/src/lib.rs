use std::io;

use eidoplot::backend::Surface;
use eidoplot::{geom, render, style};
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

    pub fn write<W>(&self, dest: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        svg::write(dest, &self.doc)
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
        assign_fill(&mut node, rect.fill.as_ref());
        assign_stroke(&mut node, rect.stroke.as_ref());
        self.doc.append(node);
        Ok(())
    }

    fn draw_path(&mut self, path: &render::Path) -> Result<(), Self::Error> {
        let mut node = svg::node::element::Path::new();
        assign_fill(&mut node, path.fill.as_ref());
        assign_stroke(&mut node, path.stroke.as_ref());
        let data = {
            let mut data = svg::node::element::path::Data::new();
            for segment in path.path.segments() {
                match segment {
                    geom::PathSegment::MoveTo(p) => {
                        data = data.move_to((p.x, p.y));
                    }
                    geom::PathSegment::LineTo(p) => {
                        data = data.line_to((p.x, p.y));
                    }
                    geom::PathSegment::Close => {
                        data = data.close();
                    }
                    _ => unreachable!(),
                }
            }
            data
        };
        node.assign("d", data);
        self.doc.append(node);
        Ok(())
    }
}

fn assign_fill<N>(node: &mut N, fill: Option<&style::Fill>)
where
    N: svg::node::Node,
{
    if let Some(fill) = fill {
        node.assign("fill", fill.color.html());
    } else {
        node.assign("fill", "none");
    }
}

fn assign_stroke<N>(node: &mut N, stroke: Option<&style::Line>)
where
    N: svg::node::Node,
{
        if let Some(stroke) = stroke {
            let w = stroke.width;
            node.assign("stroke", stroke.color.html());
            node.assign("stroke-width", w);
            match stroke.pattern {
                style::LinePattern::Solid => (),
                style::LinePattern::Dot => node.assign("stroke-dasharray", (w, w)),
                style::LinePattern::Dash(len, gap) => {
                    node.assign("stroke-dasharray", (w * len, w * gap))
                }
            }
        } else {
            node.assign("stroke", "none");
        }
}
