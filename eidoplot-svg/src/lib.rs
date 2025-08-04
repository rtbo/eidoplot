use std::io;

use eidoplot::backend::Surface;
use eidoplot::geom::Transform;
use eidoplot::style::color;
use eidoplot::{geom, render, style};

use svg::Node;
use svg::node::element;

pub struct SvgSurface {
    doc: svg::Document,
    clip_num: u32,
    group_stack: Vec<element::Group>,
}

impl SvgSurface {
    pub fn new(width: u32, height: u32) -> Self {
        let doc = svg::Document::new()
            .set("width", width)
            .set("height", height);
        SvgSurface {
            doc,
            clip_num: 0,
            group_stack: vec![],
        }
    }

    pub fn save(&self, path: &str) -> io::Result<()> {
        if !self.group_stack.is_empty() {
            panic!("Unbalanced clip stack");
        }
        svg::save(path, &self.doc)
    }

    pub fn write<W>(&self, dest: &mut W) -> io::Result<()>
    where
        W: io::Write,
    {
        if !self.group_stack.is_empty() {
            panic!("Unbalanced clip stack");
        }
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
    fn fill(&mut self, color: style::Color) -> Result<(), Self::Error> {
        let node = element::Rectangle::new()
            .set("width", "100%")
            .set("height", "100%")
            .set("fill", color.html());
        self.append_node(node);
        Ok(())
    }

    /// Draw a rectangle
    fn draw_rect(&mut self, rect: &render::Rect) -> Result<(), Self::Error> {
        let mut node = rectangle_node(&rect.rect);
        assign_fill(&mut node, rect.fill.as_ref());
        assign_stroke(&mut node, rect.stroke.as_ref());
        assign_transform(&mut node, rect.transform.as_ref());
        self.append_node(node);
        Ok(())
    }

    fn draw_path(&mut self, path: &render::Path) -> Result<(), Self::Error> {
        let mut node = element::Path::new();
        assign_fill(&mut node, path.fill.as_ref());
        assign_stroke(&mut node, path.stroke.as_ref());
        assign_transform(&mut node, path.transform.as_ref());
        node.assign("d", path_data(&path.path));
        self.append_node(node);
        Ok(())
    }

    fn push_clip_path(
        &mut self,
        path: &geom::Path,
        transform: Option<&geom::Transform>,
    ) -> Result<(), Self::Error> {
        let clip_id = self.bump_clip_id();
        let clip_id_url = format!("url(#{})", clip_id);
        let mut path_node = element::Path::new().set("d", path_data(path));
        assign_transform(&mut path_node, transform);
        let node = element::ClipPath::new()
            .set("id", clip_id.clone())
            .add(path_node);
        self.append_node(node);
        self.group_stack
            .push(element::Group::new().set("clip-path", clip_id_url));
        Ok(())
    }

    fn push_clip_rect(
        &mut self,
        rect: &geom::Rect,
        transform: Option<&geom::Transform>,
    ) -> Result<(), Self::Error> {
        let clip_id = self.bump_clip_id();
        let clip_id_url = format!("url(#{})", clip_id);
        let mut rect_node = rectangle_node(rect);
        assign_transform(&mut rect_node, transform);
        let node = element::ClipPath::new()
            .set("id", clip_id.clone())
            .add(rect_node);
        self.append_node(node);
        self.draw_rect(&render::Rect {
            rect: *rect,
            fill: None,
            stroke: Some(style::Line {
                color: color::RED,
                width: 1.0,
                pattern: style::LinePattern::Dot,
            }),
            transform: None,
        })?;
        self.group_stack
            .push(element::Group::new().set("clip-path", clip_id_url));
        Ok(())
    }

    fn pop_clip(&mut self) -> Result<(), Self::Error> {
        let g = self.group_stack.pop();
        if g.is_none() {
            panic!("Unbalanced clip stack");
        }
        self.append_node(g.unwrap());
        Ok(())
    }
}

impl SvgSurface {
    fn append_node<T>(&mut self, node: T)
    where
        T: Node,
    {
        if self.group_stack.is_empty() {
            self.doc.append(node);
        } else {
            self.group_stack.last_mut().unwrap().append(node);
        }
    }

    fn bump_clip_id(&mut self) -> String {
        self.clip_num += 1;
        format!("eidoplot-clip{}", self.clip_num)
    }
}

fn assign_transform<N>(node: &mut N, transform: Option<&geom::Transform>)
where
    N: Node,
{
    if let Some(Transform {
        sx,
        kx,
        ky,
        sy,
        tx,
        ty,
    }) = transform
    {
        node.assign(
            "transform",
            format!("matrix({sx} {kx} {ky} {sy} {tx} {ty})"),
        );
    }
}

fn assign_fill<N>(node: &mut N, fill: Option<&style::Fill>)
where
    N: Node,
{
    if let Some(fill) = fill {
        node.assign("fill", fill.color.html());
    } else {
        node.assign("fill", "none");
    }
}

fn assign_stroke<N>(node: &mut N, stroke: Option<&style::Line>)
where
    N: Node,
{
    if let Some(stroke) = stroke {
        let w = stroke.width;
        node.assign("stroke", stroke.color.html());
        node.assign("stroke-width", w);
        match stroke.pattern {
            style::LinePattern::Solid => (),
            style::LinePattern::Dot => node.assign("stroke-dasharray", (w, w)),
            style::LinePattern::Dash(style::Dash(len, gap)) => {
                node.assign("stroke-dasharray", (w * len, w * gap))
            }
        }
    } else {
        node.assign("stroke", "none");
    }
}

fn path_data(path: &geom::Path) -> element::path::Data {
    let mut data = element::path::Data::new();
    for segment in path.segments() {
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
}

fn rectangle_node(rect: &geom::Rect) -> element::Rectangle {
    element::Rectangle::new()
        .set("x", rect.x())
        .set("y", rect.y())
        .set("width", rect.width())
        .set("height", rect.height())
}
