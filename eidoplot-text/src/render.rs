use tiny_skia_path::{PathBuilder, Transform};
use ttf_parser as ttf;

use crate::font::{self, Font};
use crate::shape;

#[derive(Debug, Clone, Copy, Default)]
pub enum HorAlign {
    #[default]
    Start,
    Left,
    Center,
    End,
    Right,
}

/// Anchor where to align the text horizontally
/// By default it is a point at (X = 0)
/// Note that the transform applies on top of this anchor
#[derive(Debug, Clone, Copy)]
pub enum HorAnchor {
    /// Anchor at a X coordinate
    /// The LTR text with HorAlign::Start will start at this point and span to the right
    /// The RTL text with HorAlign::Start will start at this point and span to the left
    X(f32),
    /// Anchor in a horizontal window
    /// The following cases will be align at x_left and span to the right:
    ///     - Any text with [HorAlign::Left]
    ///     - LTR text with [HorAlign::Start]
    ///     - RTL text with [HorAlign::End]
    /// The following cases will be align at x_right and span to the left:
    ///     - Any text with [HorAlign::Right]
    ///     - LTR text with [HorAlign::End]
    ///     - RTL text with [HorAlign::Start]
    /// Centered text will be centered between x_left and x_right
    /// No check is made that the text fits in the window, and no shrinking is done
    Window { x_left: f32, x_right: f32 },
}

impl Default for HorAnchor {
    fn default() -> Self {
        HorAnchor::X(0.0)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum LineVerAlign {
    /// Align the bottom of the descender
    Bottom,
    /// Align the baseline
    #[default]
    Baseline,
    /// Align at middle of the x-height
    Middle,
    /// Align at capital height
    Hanging,
    /// Align at the top of the ascender
    Top,
}

#[derive(Debug, Clone, Copy)]
pub enum TextVerAlign {
    /// Align at the specified line
    Line(usize, LineVerAlign),
    /// Align at the top (ascender) of the first line
    Top,
    /// Align at the center, that is (top + bottom) / 2
    Center,
    /// Align at the bottom (descender) of the last line
    Bottom,
}

impl Default for TextVerAlign {
    fn default() -> Self {
        TextVerAlign::Line(0, LineVerAlign::default())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VerAnchor(pub f32);

impl Default for VerAnchor {
    fn default() -> Self {
        VerAnchor(0.0)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutOptions {
    pub hor_align: HorAlign,
    pub hor_anchor: HorAnchor,
    /// Justify the text horizontally
    /// For point anchor, justifies to the maximum line width (equals text width)
    /// For window anchor, justifies to the window width (overrules hor_align)
    pub hor_justify: bool,

    pub ver_align: TextVerAlign,
    pub ver_anchor: VerAnchor,
}

#[derive(Debug, Clone)]
pub struct RenderOptions<'a> {
    pub fill: Option<tiny_skia::Paint<'a>>,
    pub outline: Option<(tiny_skia::Paint<'a>, tiny_skia::Stroke)>,
    pub mask: Option<&'a tiny_skia::Mask>,
    pub transform: tiny_skia_path::Transform,
}

#[derive(Debug, Clone)]
struct Glyph {
    id: Option<ttf::GlyphId>,
    ts: Transform,
}

pub fn render_text(
    text_shape: &shape::Text,
    font_size: f32,
    layout_opts: &LayoutOptions,
    render_opts: &RenderOptions<'_>,
    db: &font::Database,
    pixmap: &mut tiny_skia::PixmapMut<'_>,
) {
    let lines = text_shape.lines();
    if lines.is_empty() {
        return;
    }

    let fst = &lines[0];
    let lst = &lines[lines.len() - 1];

    let mut y_cursor = -layout_opts.ver_anchor.0
        + match layout_opts.ver_align {
            TextVerAlign::Top => -fst.ascent(font_size),
            TextVerAlign::Bottom => {
                text_shape.baseline_of_line(lines.len() - 1, font_size) - lst.descent(font_size)
            }
            TextVerAlign::Center => {
                let top = -fst.ascent(font_size);
                let bottom = text_shape.baseline_of_line(lines.len() - 1, font_size)
                    - lst.descent(font_size);
                (top + bottom) / 2.0
            }
            TextVerAlign::Line(line, align) => {
                let baseline = text_shape.baseline_of_line(line, font_size);
                match align {
                    LineVerAlign::Bottom => baseline - lst.descent(font_size),
                    LineVerAlign::Baseline => baseline,
                    LineVerAlign::Middle => baseline - lst.x_height(font_size) / 2.0,
                    LineVerAlign::Hanging => baseline - lst.cap_height(font_size),
                    LineVerAlign::Top => baseline - lst.ascent(font_size),
                }
            }
        };

    let justify = if layout_opts.hor_justify {
        match layout_opts.hor_anchor {
            HorAnchor::X(..) => Some(text_shape.width(font_size)),
            HorAnchor::Window {
                x_left, x_right, ..
            } => Some(x_right - x_left),
        }
    } else {
        None
    };

    let line_align = LineAlign {
        hor_align: layout_opts.hor_align,
        hor_anchor: layout_opts.hor_anchor,
        justify,
    };

    for (i, line) in lines.iter().enumerate() {
        if i != 0 {
            y_cursor -= line.height(font_size);
        }
        render_line_at_y(
            y_cursor,
            line,
            font_size,
            line_align,
            render_opts,
            db,
            pixmap,
        );
        y_cursor -= line.gap(font_size);
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct LineAlign {
    hor_align: HorAlign,
    hor_anchor: HorAnchor,
    justify: Option<f32>,
}

fn render_line_at_y(
    y_cursor: f32,
    line: &shape::Line,
    font_size: f32,
    align: LineAlign,
    render_opts: &RenderOptions<'_>,
    db: &font::Database,
    pixmap: &mut tiny_skia::PixmapMut<'_>,
) {
    let width = line.width(font_size);

    let (width, justify_gap) = match align.justify {
        Some(justify) => {
            if justify <= width {
                (width, None)
            } else {
                let adv_count = line.glyphs().iter().filter(|g| g.has_x_advance()).count();
                let gap = if adv_count > 1 {
                    (justify - width) / (adv_count as f32 - 1.0)
                } else {
                    0.0
                };
                (justify, Some(gap))
            }
        }
        None => (width, None),
    };

    let (left, right) = match align.hor_anchor {
        HorAnchor::X(x) => (x, x),
        HorAnchor::Window { x_left, x_right } => (x_left, x_right),
    };

    let mut x_cursor = match (align.hor_align, line.rtl()) {
        (HorAlign::Start, false) | (HorAlign::End, true) | (HorAlign::Left, _) => left,
        (HorAlign::Center, _) => (left + right) / 2.0 - width / 2.0,
        (HorAlign::Start, true) | (HorAlign::End, false) | (HorAlign::Right, _) => right - width,
    };

    let mut y_cursor = y_cursor;

    // grouping by font-id in order to avoid loading the same font on every glyph
    let mut runs = Vec::new();
    let mut run_start = 0;
    let mut glyphs_buf = Vec::with_capacity(line.glyphs().len());
    let mut font_id = None;

    for (gi, sh_gl) in line.glyphs().iter().enumerate() {
        let scale = sh_gl.scale(font_size);
        let scale_ts = Transform::from_scale(scale, scale);

        let x = x_cursor + sh_gl.x_offset(font_size);
        let y = y_cursor + sh_gl.y_offset(font_size);
        let pos_ts = Transform::from_translate(x, y);

        let gl = Glyph {
            id: sh_gl.id(),
            ts: scale_ts.post_concat(pos_ts),
        };
        if let Some(font_id) = font_id {
            if sh_gl.font_id() != font_id {
                runs.push((run_start, gi, font_id));
                run_start = gi;
            }
        }

        glyphs_buf.push(gl);
        font_id = Some(sh_gl.font_id());

        x_cursor += sh_gl.x_advance(font_size);
        y_cursor += sh_gl.y_advance(font_size);
        if let Some(jg) = justify_gap {
            if sh_gl.has_x_advance() {
                x_cursor += jg;
            }
        }
    }
    if run_start < line.glyphs().len() {
        runs.push((run_start, line.glyphs().len(), font_id.unwrap()));
    }

    for r in runs {
        render_glyphs(
            &glyphs_buf[r.0..r.1],
            render_opts,
            r.2,
            line.font(),
            db,
            pixmap,
        );
    }
}

fn render_glyphs(
    glyphs: &[Glyph],
    render_opts: &RenderOptions<'_>,
    font_id: font::ID,
    font: &Font,
    db: &font::Database,
    pixmap: &mut tiny_skia::PixmapMut<'_>,
) {
    println!("rendering with {:?}", db.face(font_id).unwrap());

    db.with_face_data(font_id, |data, index| {
        let mut face = ttf::Face::parse(data, index).unwrap();
        font::apply_variations(&mut face, font);

        // the path builder for the entire string
        let mut str_pb = PathBuilder::new();
        // the path builder for each glyph
        let mut gl_pb = PathBuilder::new();

        for gl in glyphs {
            let Some(glyph_id) = gl.id else {
                continue;
            };

            {
                let mut builder = Outliner(&mut gl_pb);
                face.outline_glyph(glyph_id, &mut builder);
            }

            if let Some(path) = gl_pb.finish() {
                let path = path.transform(gl.ts).unwrap();
                str_pb.push_path(&path);

                gl_pb = path.clear();
            } else {
                gl_pb = PathBuilder::new();
            }
        }

        if let Some(path) = str_pb.finish() {
            let path = path.transform(Transform::from_scale(1.0, -1.0)).unwrap();
            if let Some(paint) = render_opts.fill.as_ref() {
                pixmap.fill_path(
                    &path,
                    &paint,
                    tiny_skia::FillRule::Winding,
                    render_opts.transform,
                    render_opts.mask,
                );
            }
            if let Some((paint, stroke)) = render_opts.outline.as_ref() {
                pixmap.stroke_path(
                    &path,
                    &paint,
                    &stroke,
                    render_opts.transform,
                    render_opts.mask,
                );
            }
        }
    })
    .unwrap();
}

struct Outliner<'a>(&'a mut PathBuilder);

impl ttf::OutlineBuilder for Outliner<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.quad_to(x1, y1, x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.cubic_to(x1, y1, x2, y2, x, y);
    }

    fn close(&mut self) {
        self.0.close();
    }
}
