use std::io;

use eidoplot::geom::Transform;
use eidoplot::render::Surface;
use eidoplot::style::ColorU8;
use eidoplot::{geom, render};
use eidoplot_text::{self as text, line, rich};
use svg::Node;
use svg::node::element;
use text::font;

pub struct SvgSurface {
    doc: svg::Document,
    clip_num: u32,
    _node_num: u32,
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
            _node_num: 0,
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

    fn draw_line_text(&mut self, rtext: &render::LineText) -> Result<(), render::Error> {
        let text = rtext.text;
        let (align, ver_align) = text.align();

        let mut node = element::Text::new(text.text())
            .set("text-rendering", "optimizeLegibility")
            .set("text-anchor", line_text_anchor(align, text.main_dir()));

        let (db, yshift) = dominant_baseline(ver_align, Some(text.metrics()), text.font_size());
        node.assign("dominant-baseline", db);

        let shift = Transform::from_translate(0.0, yshift);
        let transform = rtext.transform.post_concat(shift);

        assign_font(&mut node, text.font(), text.font_size());
        assign_fill(&mut node, Some(&rtext.fill));
        assign_transform(&mut node, Some(&transform));

        self.append_node(node);
        Ok(())
    }

    fn draw_rich_text(&mut self, text: &render::RichText) -> Result<(), render::Error> {
        match text.text.layout() {
            rich::Layout::Horizontal(align, _, _) => self.draw_rich_text_hor(text, align),
            rich::Layout::Vertical(align, hor_align, _, progression, _) => {
                self.draw_rich_text_ver(text, align, hor_align, progression)
            }
        }
    }

    fn draw_text(&mut self, text: &render::Text) -> Result<(), render::Error> {
        let mut node = element::Text::new(text.text)
            .set("text-rendering", "optimizeLegibility")
            // have to assume LTR as no shaping is done
            .set(
                "text-anchor",
                line_text_anchor(text.align.0, text::ScriptDir::LeftToRight),
            );

        let (db, yshift) = dominant_baseline(text.align.1, None, text.font_size);
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

    fn _bump_node_id(&mut self) -> String {
        self._node_num += 1;
        format!("eidoplot-node{}", self._node_num)
    }

    fn draw_rich_text_hor(
        &mut self,
        text: &render::RichText,
        align: rich::Align,
    ) -> Result<(), render::Error> {
        let mut node =
            element::Text::new(String::new()).set("text-rendering", "optimizeLegibility");

        let whole_txt = text.text.text();

        let mut dy = 0.0;

        for line in text.text.lines().iter() {
            let mut line_node = element::TSpan::new(String::new())
                .set("text-anchor", rich_text_anchor(align, line.main_dir()))
                .set("x", 0.0);
            if dy != 0.0 {
                line_node.assign("dy", dy);
            }

            for shape in line.shapes() {
                let mut shape_node = element::TSpan::new(String::new());

                for (idx, span) in shape.spans().iter().enumerate() {
                    if idx == 0 {
                        assign_font(
                            &mut shape_node,
                            span.props().font(),
                            span.props().font_size(),
                        );
                    }
                    let span_txt = &whole_txt[span.start()..span.end()];
                    let mut span_node = element::TSpan::new(span_txt);
                    let paint = span.props().fill().map(|c| {
                        render::Paint::Solid(ColorU8::from_rgba(
                            c.red(),
                            c.green(),
                            c.blue(),
                            c.alpha(),
                        ))
                    });
                    assign_fill(&mut span_node, paint.as_ref());
                    shape_node.append(span_node);
                }

                line_node.append(shape_node);
            }
            node.append(line_node);

            dy += line.total_height();
        }

        let yshift = rich_text_hor_yshift(&text.text);
        let transform = text
            .transform
            .post_concat(Transform::from_translate(0.0, yshift));
        assign_transform(&mut node, Some(&transform));

        self.append_node(node);
        Ok(())
    }

    fn draw_rich_text_ver(
        &mut self,
        _text: &render::RichText,
        _align: rich::Align,
        _hor_align: rich::HorAlign,
        progression: rich::VerProgression,
    ) -> Result<(), render::Error> {
        let writing_mode = match progression {
            rich::VerProgression::LTR => "vertical-lr",
            rich::VerProgression::RTL => "vertical-rl",
            _ => unreachable!(),
        };
        let _text_style = format!(
            "writing-mode: {};\ntext-orientation: upright;\n",
            writing_mode
        );
        todo!()
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

fn font_style(style: text::font::Style) -> &'static str {
    match style {
        text::font::Style::Normal => "normal",
        text::font::Style::Italic => "italic",
        text::font::Style::Oblique => "oblique",
    }
}

fn font_stretch(width: text::font::Width) -> &'static str {
    match width {
        text::font::Width::UltraCondensed => "ultra-condensed",
        text::font::Width::ExtraCondensed => "extra-condensed",
        text::font::Width::Condensed => "condensed",
        text::font::Width::SemiCondensed => "semi-condensed",
        text::font::Width::Normal => "normal",
        text::font::Width::SemiExpanded => "semi-expanded",
        text::font::Width::Expanded => "expanded",
        text::font::Width::ExtraExpanded => "extra-expanded",
        text::font::Width::UltraExpanded => "ultra-expanded",
    }
}

fn line_text_anchor(align: line::Align, direction: text::ScriptDir) -> &'static str {
    match (align, direction) {
        (line::Align::Start, _) => "start",
        (line::Align::Center, _) => "middle",
        (line::Align::End, _) => "end",
        (line::Align::Left, text::ScriptDir::LeftToRight) => "start",
        (line::Align::Left, text::ScriptDir::RightToLeft) => "end",
        (line::Align::Right, text::ScriptDir::LeftToRight) => "end",
        (line::Align::Right, text::ScriptDir::RightToLeft) => "start",
    }
}

fn rich_text_anchor(align: rich::Align, direction: rustybuzz::Direction) -> &'static str {
    match (align, direction) {
        (rich::Align::Start, _) => "start",
        (rich::Align::Center, _) => "middle",
        (rich::Align::End, _) => "end",
        (rich::Align::Left, rustybuzz::Direction::LeftToRight) => "start",
        (rich::Align::Left, rustybuzz::Direction::RightToLeft) => "end",
        (rich::Align::Right, rustybuzz::Direction::LeftToRight) => "end",
        (rich::Align::Right, rustybuzz::Direction::RightToLeft) => "start",
        (rich::Align::Justify(_), _) => todo!("justified text for SVG"),
        _ => unreachable!("anchor not relevant for vertical text"),
    }
}

fn dominant_baseline(
    align: line::VerAlign,
    metrics: Option<text::font::ScaledMetrics>,
    font_size: f32,
) -> (&'static str, f32) {
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
        line::VerAlign::Middle => ("middle", 0.0),
        line::VerAlign::Hanging => ("hanging", 0.0),
        line::VerAlign::Baseline => ("alphabetic", 0.0),
        line::VerAlign::Top => (
            "hanging",
            metrics
                .map(|m| m.ascent - m.cap_height)
                .unwrap_or(TOP_FACTOR * font_size),
        ),
        line::VerAlign::Bottom => (
            "alphabetic",
            metrics
                .map(|m| m.descent)
                .unwrap_or(BOTTOM_FACTOR * font_size),
        ),
    }
}

trait Lines {
    fn baseline(&self, idx: usize) -> f32;
}

impl Lines for [rich::LineSpan] {
    fn baseline(&self, idx: usize) -> f32 {
        let mut h = 0.0;
        let mut l = 0;
        while l < idx {
            h += self[l].total_height();
            l += 1;
        }
        h
    }
}

fn rich_text_hor_yshift(text: &rich::RichText) -> f32 {
    // multiple lines in SVG is a bit tricky.
    // so we don't use dominant-baseline at all and we apply
    // a vertical shift from the font face.
    // It means that the shift is relative to the baseline of the first line

    if text.lines().is_empty() {
        return 0.0;
    }
    let rich::Layout::Horizontal(_, ver_align, _) = text.layout() else {
        unreachable!()
    };

    let lines = text.lines();
    let lines_len = lines.len();

    // y-cursor must be placed at the baseline of the first line
    match ver_align {
        rich::VerAlign::Top => lines[0].ascent(),
        rich::VerAlign::Bottom => lines[lines_len - 1].descent() - lines.baseline(lines_len - 1),
        rich::VerAlign::Center => {
            let top = lines[0].ascent();
            let bottom = lines[lines_len - 1].descent() - lines.baseline(lines_len - 1);
            (top + bottom) / 2.0
        }
        rich::VerAlign::Line(line, align) => {
            let baseline = lines.baseline(line);
            let lst_line = &lines[lines_len - 1];
            match align {
                line::VerAlign::Bottom => lst_line.descent() - baseline,
                line::VerAlign::Baseline => -baseline,
                line::VerAlign::Middle => lst_line.x_height() / 2.0 - baseline,
                line::VerAlign::Hanging => lst_line.cap_height() - baseline,
                line::VerAlign::Top => lst_line.ascent() - baseline,
            }
        }
    }
}
