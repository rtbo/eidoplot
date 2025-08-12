use std::io;

use eidoplot::geom::Transform;
use eidoplot::render::Surface;
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

    /// Prepare the surface for drawing, with the given width and height in plot units
    fn prepare(&mut self, size: geom::Size) -> Result<(), render::Error> {
        self.doc
            .assign("viewBox", (0, 0, size.width(), size.height()));
        Ok(())
    }

    /// Fill the entire surface with the given color
    fn fill(&mut self, fill: style::Fill) -> Result<(), render::Error> {
        let mut node = element::Rectangle::new()
            .set("width", "100%")
            .set("height", "100%");
        match fill {
            style::Fill::Solid(color) => node.assign("fill", color.html()),
        }
        self.append_node(node);
        Ok(())
    }

    /// Draw a rectangle
    fn draw_rect(&mut self, rect: &render::Rect) -> Result<(), render::Error> {
        let mut node = rectangle_node(&rect.rect);
        assign_fill(&mut node, rect.fill.as_ref());
        assign_stroke(&mut node, rect.stroke.as_ref());
        assign_transform(&mut node, rect.transform);
        self.append_node(node);
        Ok(())
    }

    fn draw_path(&mut self, path: &render::Path) -> Result<(), render::Error> {
        let mut node = element::Path::new();
        assign_fill(&mut node, path.fill.as_ref());
        assign_stroke(&mut node, path.stroke.as_ref());
        assign_transform(&mut node, path.transform);
        node.assign("d", path_data(path.path));
        self.append_node(node);
        Ok(())
    }

    fn draw_text(&mut self, text: &render::Text) -> Result<(), render::Error> {
        let font = &text.font;
        let color = match text.fill {
            style::Fill::Solid(color) => color,
        };

        let mut node = element::Text::new(text.text)
            .set("font-family", font.family().as_str())
            .set("font-size", font.size())
            .set("font-weight", font.weight().0)
            .set("font-style", font_style(font.style()))
            .set("font-stretch", font_stretch(font.width()))
            .set("fill", color.html())
            .set("x", text.anchor.pos.x())
            .set("y", text.anchor.pos.y())
            .set("text-rendering", "optimizeLegibility")
            .set("text-anchor", text_anchor(text.anchor.align))
            .set("dominant-baseline", dominant_baseline(text.anchor.baseline));

        assign_transform(&mut node, text.transform);
        self.append_node(node);
        Ok(())
    }

    fn push_clip(&mut self, clip: &render::Clip) -> Result<(), render::Error> {
        let clip_id = self.bump_clip_id();
        let clip_id_url = format!("url(#{})", clip_id);
        let mut path_node = element::Path::new().set("d", path_data(&clip.path));
        assign_transform(&mut path_node, clip.transform);
        let node = element::ClipPath::new()
            .set("id", clip_id.clone())
            .add(path_node);
        self.append_node(node);
        self.group_stack
            .push(element::Group::new().set("clip-path", clip_id_url));
        Ok(())
    }

    fn pop_clip(&mut self) -> Result<(), render::Error> {
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
    if let Some(style::Fill::Solid(color)) = fill {
        node.assign("fill", color.html());
        if let Some(opacity) = color.opacity() {
            node.assign("fill-opacity", opacity);
        }
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
        if let Some(opacity) = stroke.color.opacity() {
            node.assign("stroke-opacity", opacity);
        }
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

fn font_style(style: style::font::Style) -> &'static str {
    match style {
        style::font::Style::Normal => "normal",
        style::font::Style::Italic => "italic",
        style::font::Style::Oblique => "oblique",
    }
}

fn font_stretch(width: style::font::Width) -> &'static str {
    match width {
        style::font::Width::UltraCondensed => "ultra-condensed",
        style::font::Width::ExtraCondensed => "extra-condensed",
        style::font::Width::Condensed => "condensed",
        style::font::Width::SemiCondensed => "semi-condensed",
        style::font::Width::Normal => "normal",
        style::font::Width::SemiExpanded => "semi-expanded",
        style::font::Width::Expanded => "expanded",
        style::font::Width::ExtraExpanded => "extra-expanded",
        style::font::Width::UltraExpanded => "ultra-expanded",
    }
}

fn text_anchor(align: render::TextAlign) -> &'static str {
    match align {
        render::TextAlign::Start => "start",
        render::TextAlign::Center => "middle",
        render::TextAlign::End => "end",
    }
}

fn dominant_baseline(baseline: render::TextBaseline) -> &'static str {
    match baseline {
        render::TextBaseline::Base => "alphabetic",
        render::TextBaseline::Center => "middle",
        render::TextBaseline::Hanging => "hanging",
    }
}
