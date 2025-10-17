use super::{RichText, ShapeSpan};
use crate::{BBox, font, fontdb};

use ttf_parser as ttf;

impl ShapeSpan {
    fn font(&self) -> &font::Font {
        &self.spans.first().unwrap().props.font
    }
}

pub fn render_rich_text(
    text: &RichText,
    fontdb: &fontdb::Database,
    transform: tiny_skia_path::Transform,
    mask: Option<&tiny_skia::Mask>,
    pixmap: &mut tiny_skia::PixmapMut<'_>,
) -> Result<(), ttf::FaceParsingError> {
    let mut span_builder = tiny_skia_path::PathBuilder::new();
    let mut glyph_builder = tiny_skia_path::PathBuilder::new();

    for line in &text.lines {
        for shape in &line.shapes {
            (glyph_builder, span_builder) = fontdb
                .with_face_data(shape.face_id, |data, index| {
                    let mut face = ttf::Face::parse(data, index).unwrap();
                    font::apply_ttf_variations(&mut face, shape.font());

                    // TODO: get span bbox and render underline and strikeout lines

                    for span in &shape.spans {
                        for glyph in shape
                            .glyphs
                            .iter()
                            .filter(|g| g.cluster >= span.start && g.cluster < span.end)
                        {
                            {
                                let mut builder = Outliner(&mut glyph_builder);
                                face.outline_glyph(glyph.id, &mut builder);
                            }

                            if let Some(path) = glyph_builder.finish() {
                                let path = path.transform(glyph.ts).unwrap();
                                span_builder.push_path(&path);

                                glyph_builder = path.clear();
                            } else {
                                glyph_builder = tiny_skia::PathBuilder::new();
                            }
                        }

                        if span.props.underline {
                            let line = shape.metrics.uline;
                            let path = line_path(span.bbox, shape.y_baseline, line, glyph_builder);
                            span_builder.push_path(&path);
                            glyph_builder = path.clear();
                        }
                        if span.props.strikeout {
                            let line = shape.metrics.strikeout;
                            let path = line_path(span.bbox, shape.y_baseline, line, glyph_builder);
                            span_builder.push_path(&path);
                            glyph_builder = path.clear();
                        }

                        if let Some(path) = span_builder.finish() {
                            if let Some(c) = span.props.fill.as_ref() {
                                let mut paint = tiny_skia::Paint::default();
                                paint.set_color_rgba8(c.r, c.g, c.b, c.a);
                                pixmap.fill_path(
                                    &path,
                                    &paint,
                                    tiny_skia::FillRule::Winding,
                                    transform,
                                    mask,
                                );
                            }
                            if let Some((c, thickness)) = span.props.outline.as_ref() {
                                let mut paint = tiny_skia::Paint::default();
                                paint.set_color_rgba8(c.r, c.g, c.b, c.a);
                                let mut stroke = tiny_skia::Stroke::default();
                                stroke.width = *thickness;
                                pixmap.stroke_path(&path, &paint, &stroke, transform, mask);
                            }
                            span_builder = path.clear();
                        } else {
                            span_builder = tiny_skia_path::PathBuilder::new();
                        }
                    }

                    (glyph_builder, span_builder)
                })
                .unwrap();
        }
    }

    Ok(())
}

fn line_path(
    bbox: BBox,
    y_baseline: f32,
    line: font::ScaledLineMetrics,
    mut builder: tiny_skia_path::PathBuilder,
) -> tiny_skia::Path {
    // there is no y-flip transform on this one
    builder.move_to(bbox.left, y_baseline - line.position);
    builder.line_to(bbox.right, y_baseline - line.position);
    builder.line_to(bbox.right, y_baseline - line.position + line.thickness);
    builder.line_to(bbox.left, y_baseline - line.position + line.thickness);
    builder.close();
    builder.finish().unwrap()
}

struct Outliner<'a>(&'a mut tiny_skia_path::PathBuilder);

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
