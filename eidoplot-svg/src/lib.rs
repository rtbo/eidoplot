use std::io;

use eidoplot::geom::Transform;
use eidoplot::render::Surface;
use eidoplot::{geom, render, style};
use eidoplot_text as text;
use svg::Node;
use svg::node::element;
use text::font;

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

    pub fn save_svg<P: AsRef<std::path::Path>>(&self, path: P) -> io::Result<()> {
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
    fn fill(&mut self, fill: render::Paint) -> Result<(), render::Error> {
        let mut node = element::Rectangle::new()
            .set("width", "100%")
            .set("height", "100%");
        match fill {
            render::Paint::Solid(color) => node.assign("fill", color.html()),
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
        let mut node = element::Text::new(text.text)
            .set("text-rendering", "optimizeLegibility")
            // have to assume LTR as no shaping is done
            .set(
                "text-anchor",
                text_anchor(text.options.hor_align, text::Direction::LTR),
            );

        let (db, yshift) = dominant_baseline(text.options.ver_align, None, text.font_size);
        node.assign("dominant-baseline", db);

        let shift = Transform::from_translate(0.0, yshift);
        let transform = text
            .transform
            .map(|t| t.post_concat(shift))
            .unwrap_or(shift);

        assign_font(&mut node, &text.font, text.font_size);
        assign_fill(&mut node, Some(&text.fill));
        assign_transform(&mut node, Some(&transform));

        self.append_node(node);

        Ok(())
    }

    fn draw_text_layout(&mut self, text: &render::TextLayout) -> Result<(), render::Error> {
        let layout = text.layout;
        let options = layout.options();

        let mut node = element::Text::new(layout.text())
            .set("text-rendering", "optimizeLegibility")
            .set(
                "text-anchor",
                text_anchor(options.hor_align, layout.direction()),
            );

        let (db, yshift) = dominant_baseline(
            options.ver_align,
            Some(layout.metrics()),
            layout.font_size(),
        );
        node.assign("dominant-baseline", db);

        let shift = Transform::from_translate(0.0, yshift);
        let transform = text
            .transform
            .map(|t| t.post_concat(shift))
            .unwrap_or(shift);

        assign_font(&mut node, layout.font(), layout.font_size());
        assign_fill(&mut node, Some(&text.fill));
        assign_transform(&mut node, Some(&transform));

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
            format!("matrix({sx} {ky} {kx} {sy} {tx} {ty})"),
        );
    }
}

fn assign_fill<N>(node: &mut N, fill: Option<&render::Paint>)
where
    N: Node,
{
    if let Some(render::Paint::Solid(color)) = fill {
        node.assign("fill", color.html());
        if let Some(opacity) = color.opacity() {
            node.assign("fill-opacity", opacity);
        }
    } else {
        node.assign("fill", "none");
    }
}

fn assign_stroke<N>(node: &mut N, stroke: Option<&render::Stroke>)
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
            render::LinePattern::Solid => (),
            render::LinePattern::Dash(dash) => {
                let array: Vec<f32> = dash.iter().map(|d| d * w).collect();
                node.assign("stroke-dasharray", array)
            }
        }
    } else {
        node.assign("stroke", "none");
    }
}

fn assign_font<N>(node: &mut N, font: &text::Font, font_size: f32)
where
    N: Node,
{
    let family = font::font_families_to_string(font.families());
    node.assign("font-size", font_size);
    node.assign("font-family", family.as_str());
    node.assign("font-weight", font.weight().0);
    node.assign("font-style", font_style(font.style()));
    node.assign("font-stretch", font_stretch(font.width()));
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
            geom::PathSegment::QuadTo(p1, p2) => {
                data = data.quadratic_curve_to((p1.x, p1.y, p2.x, p2.y));
            }
            geom::PathSegment::CubicTo(p1, p2, p3) => {
                data = data.cubic_curve_to((p1.x, p1.y, p2.x, p2.y, p3.x, p3.y));
            }
            geom::PathSegment::Close => {
                data = data.close();
            }
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

fn text_anchor(align: text::HorAlign, direction: text::Direction) -> &'static str {
    match (align, direction) {
        (text::HorAlign::Start, _) => "start",
        (text::HorAlign::Center, _) => "middle",
        (text::HorAlign::End, _) => "end",
        (text::HorAlign::Left, text::Direction::LTR) => "start",
        (text::HorAlign::Left, text::Direction::RTL) => "end",
        (text::HorAlign::Right, text::Direction::LTR) => "end",
        (text::HorAlign::Right, text::Direction::RTL) => "start",
    }
}

fn dominant_baseline(
    align: text::VerAlign,
    metrics: Option<text::ScaledMetrics>,
    font_size: f32,
) -> (&'static str, f32) {
    if let text::VerAlign::Line(lidx, _) = align {
        assert!(lidx == 0, "Only single line is supported");
    }

    // text-top and text-bottom don't work too well,
    // so instead we apply hanging and alphabetic,
    // with a vertical shift from the font face if available, or hard-coded from the font_size

    // the following factors work for Noto-Sans.

    const TOP_FACTOR: f32 = 0.355;
    const BOTTOM_FACTOR: f32 = -0.293;

    // the following block can be activated to print factors for other fonts
    #[cfg(false)]
    if let Some(m) = metrics {
        let top_factor = (m.ascent - m.cap_height) / font_size;
        let bottom_factor = m.descent / font_size;
        println!("top factor = {top_factor}");
        println!("bottom factor = {bottom_factor}");
    }

    match align {
        text::VerAlign::Center => ("center", 0.0),
        //text::VerAlign::Top => ("text-top", 0.0),
        text::VerAlign::Top => (
            "hanging",
            metrics
                .map(|m| m.ascent - m.cap_height)
                .unwrap_or(TOP_FACTOR * font_size),
        ),
        //text::VerAlign::Bottom => "text-bottom",
        text::VerAlign::Bottom => (
            "alphabetic",
            metrics
                .map(|m| m.descent)
                .unwrap_or(BOTTOM_FACTOR * font_size),
        ),
        // text::VerAlign::Line(_, text::LineVerAlign::Top) => "text-top",
        text::VerAlign::Line(_, text::LineVerAlign::Top) => (
            "hanging",
            metrics
                .map(|m| m.ascent - m.cap_height)
                .unwrap_or(TOP_FACTOR * font_size),
        ),
        text::VerAlign::Line(_, text::LineVerAlign::Hanging) => ("hanging", 0.0),
        text::VerAlign::Line(_, text::LineVerAlign::Middle) => ("middle", 0.0),
        text::VerAlign::Line(_, text::LineVerAlign::Baseline) => ("alphabetic", 0.0),
        // text::VerAlign::Line(_, text::LineVerAlign::Bottom) => ("text-bottom", 0.0),
        text::VerAlign::Line(_, text::LineVerAlign::Bottom) => (
            "alphabetic",
            metrics
                .map(|m| m.descent)
                .unwrap_or(BOTTOM_FACTOR * font_size),
        ),
    }
}
