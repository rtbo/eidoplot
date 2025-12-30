use std::path::Path;
use std::{fmt, io};

use eidoplot::geom::{self, Transform};
use eidoplot::render::{self, Surface};
use eidoplot::{Style, drawing, style};
use svg::Node;
use svg::node::element;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Drawing(drawing::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<drawing::Error> for Error {
    fn from(err: drawing::Error) -> Self {
        Error::Drawing(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Drawing(err) => write!(f, "Drawing error: {}", err),
        }
    }
}

impl std::error::Error for Error {}

/// Parameters needed for saving a [`drawing::Figure`] as SVG
#[derive(Debug, Clone)]
pub struct DrawingParams<T, SP> {
    pub style: Style<T, SP>,
    pub scale: f32,
}

impl<T, SP> Default for DrawingParams<T, SP>
where
    T: style::Theme + Default,
    SP: style::series::Palette + Default,
{
    fn default() -> Self {
        Self {
            style: Style {
                theme: T::default(),
                palette: SP::default(),
            },
            scale: 1.0,
        }
    }
}

pub trait SaveSvg {
    fn save_svg<P, T, SP>(&self, path: P, params: DrawingParams<T, SP>) -> Result<(), Error>
    where
        P: AsRef<Path>,
        T: style::Theme,
        SP: style::series::Palette;
}

impl SaveSvg for drawing::Figure {
    fn save_svg<P, T, SP>(&self, path: P, params: DrawingParams<T, SP>) -> Result<(), Error>
    where
        P: AsRef<Path>,
        T: style::Theme,
        SP: style::series::Palette,
    {
        let size = self.size();
        let witdth = (size.width() * params.scale) as u32;
        let height = (size.height() * params.scale) as u32;

        let mut surface = SvgSurface::new(witdth, height);

        self.draw(&mut surface, &params.style);
        surface.save_svg(path)?;
        Ok(())
    }
}

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
    fn prepare(&mut self, size: geom::Size) {
        self.doc
            .assign("viewBox", (0, 0, size.width(), size.height()));
    }

    /// Fill the entire surface with the given color
    fn fill(&mut self, fill: render::Paint) {
        let mut node = element::Rectangle::new()
            .set("width", "100%")
            .set("height", "100%");
        match fill {
            render::Paint::Solid(color) => node.assign("fill", color.html()),
        }
        self.append_node(node);
    }

    /// Draw a rectangle
    fn draw_rect(&mut self, rect: &render::Rect) {
        let mut node = rectangle_node(&rect.rect);
        assign_fill(&mut node, rect.fill.as_ref());
        assign_stroke(&mut node, rect.stroke.as_ref());
        assign_transform(&mut node, rect.transform);
        self.append_node(node);
    }

    fn draw_path(&mut self, path: &render::Path) {
        let mut node = element::Path::new();
        assign_fill(&mut node, path.fill.as_ref());
        assign_stroke(&mut node, path.stroke.as_ref());
        assign_transform(&mut node, path.transform);
        node.assign("d", path_data(path.path));
        self.append_node(node);
    }

    fn push_clip(&mut self, clip: &render::Clip) {
        let clip_id = self.bump_clip_id();
        let clip_id_url = format!("url(#{})", clip_id);
        let mut rect_node = rectangle_node(&clip.rect);
        assign_transform(&mut rect_node, clip.transform);
        let node = element::ClipPath::new()
            .set("id", clip_id.clone())
            .add(rect_node);
        self.append_node(node);
        self.group_stack
            .push(element::Group::new().set("clip-path", clip_id_url));
    }

    fn pop_clip(&mut self) {
        let g = self.group_stack.pop();
        if g.is_none() {
            panic!("Unbalanced clip stack");
        }
        self.append_node(g.unwrap());
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

    // fn draw_rich_text_hor(
    //     &mut self,
    //     text: &render::RichText,
    //     align: rich::Align,
    // ) -> Result<(), render::Error> {
    //     let mut node =
    //         element::Text::new(String::new()).set("text-rendering", "optimizeLegibility");

    //     let whole_txt = text.text.text();

    //     let mut dy = 0.0;
    //     let mut last_height = 0.0;

    //     for line in text.text.lines().iter() {
    //         let mut line_node = element::TSpan::new(String::new())
    //             .set("text-anchor", rich_text_anchor(align, line.main_dir()))
    //             .set("x", 0.0);

    //         let this_height = line.total_height();
    //         if dy != 0.0 {
    //             dy += this_height - last_height;
    //             line_node.assign("dy", dy);
    //         }

    //         for shape in line.shapes() {
    //             let mut shape_node = element::TSpan::new(String::new());

    //             for (idx, span) in shape.spans().iter().enumerate() {
    //                 if idx == 0 {
    //                     assign_font(
    //                         &mut shape_node,
    //                         span.props().font(),
    //                         span.props().font_size(),
    //                     );
    //                 }
    //                 let span_txt = &whole_txt[span.start()..span.end()];
    //                 let mut span_node = element::TSpan::new(span_txt);
    //                 let paint = span.props().fill().map(|c| {
    //                     render::Paint::Solid(ColorU8::from_rgba(
    //                         c.red(),
    //                         c.green(),
    //                         c.blue(),
    //                         c.alpha(),
    //                     ))
    //                 });
    //                 assign_fill(&mut span_node, paint.as_ref());
    //                 shape_node.append(span_node);
    //             }

    //             line_node.append(shape_node);
    //         }
    //         node.append(line_node);

    //         last_height = this_height;
    //         dy += last_height;
    //     }

    //     let yshift = rich_text_hor_yshift(&text.text);
    //     let transform = text
    //         .transform
    //         .pre_concat(Transform::from_translate(0.0, yshift));
    //     assign_transform(&mut node, Some(&transform));

    //     self.append_node(node);
    //     Ok(())
    // }

    // fn draw_rich_text_ver(
    //     &mut self,
    //     _text: &render::RichText,
    //     _align: rich::Align,
    //     _hor_align: rich::HorAlign,
    //     progression: rich::VerProgression,
    // ) -> Result<(), render::Error> {
    //     let writing_mode = match progression {
    //         rich::VerProgression::LTR => "vertical-lr",
    //         rich::VerProgression::RTL => "vertical-rl",
    //         _ => unreachable!(),
    //     };
    //     let _text_style = format!(
    //         "writing-mode: {};\ntext-orientation: upright;\n",
    //         writing_mode
    //     );
    //     todo!()
    // }
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
